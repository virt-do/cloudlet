use std::net::{IpAddr, Ipv4Addr};

use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};

use super::xx_netmask_width;

#[derive(Clone)]
pub struct Bridge {
    name: String,
    handle: Handle,
}

impl Bridge {
    pub fn new(name: String) -> Self {
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        let br = Self { name, handle };
        br.create_bridge_if_not_exist();

        br
    }

    fn create_bridge_if_not_exist(&self) {
        futures::executor::block_on(async {
            let mut bridge_names = self
                .handle
                .link()
                .get()
                .match_name(self.name.clone())
                .execute();

            let _ = match bridge_names.try_next().await {
                Ok(_) => Ok(()),
                Err(_) => self
                    .handle
                    .link()
                    .add()
                    .bridge(self.name.clone())
                    .execute()
                    .await
                    .map_err(|_| Error::RequestFailed),
            };
        });
    }

    pub fn set_addr(&self, addr: Ipv4Addr, netmask: Ipv4Addr) {
        futures::executor::block_on(async {
            let mut bridge_names = self
                .handle
                .link()
                .get()
                .match_name(self.name.clone())
                .execute();

            let bridge_index = match bridge_names.try_next().await {
                Ok(Some(link)) => link.header.index,
                Ok(None) => panic!(),
                Err(_) => panic!(),
            };

            let prefix_len = xx_netmask_width(netmask.octets());

            let _ = self
                .handle
                .address()
                .add(bridge_index, IpAddr::V4(addr), prefix_len)
                .execute()
                .await
                .map_err(|_| Error::RequestFailed);
        });
    }

    pub fn set_up(&self) {
        futures::executor::block_on(async {
            let mut bridge_names = self
                .handle
                .link()
                .get()
                .match_name(self.name.clone())
                .execute();

            let bridge_index = match bridge_names.try_next().await {
                Ok(Some(link)) => link.header.index,
                Ok(None) => panic!(),
                Err(_) => panic!(),
            };

            let _ = self
                .handle
                .link()
                .set(bridge_index)
                .up()
                .execute()
                .await
                .map_err(|_| Error::RequestFailed);
        });
    }

    pub fn attach_link(&self, link_name: String) {
        futures::executor::block_on(async {
            let mut link_names = self
                .handle
                .link()
                .get()
                .match_name(link_name.clone())
                .execute();
            let mut master_names = self
                .handle
                .link()
                .get()
                .match_name(self.name.clone())
                .execute();

            let link_index = match link_names.try_next().await {
                Ok(Some(link)) => link.header.index,
                Ok(None) => panic!(),
                Err(_) => panic!(),
            };
            let master_index = match master_names.try_next().await {
                Ok(Some(link)) => link.header.index,
                Ok(None) => panic!(),
                Err(_) => panic!(),
            };

            let _ = self
                .handle
                .link()
                .set(link_index)
                .controller(master_index)
                .execute()
                .await
                .map_err(|_| Error::RequestFailed);
        });
    }
}
