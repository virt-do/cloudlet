use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};

pub fn host_bridge(tap_name: String, bridge_name: String) {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    futures::executor::block_on(async {
        let _ = create_bridge_if_not_exist(handle.clone(), bridge_name.clone()).await;
        let _ = attach_link_to_bridge(handle, tap_name, bridge_name).await;
    })
}

async fn create_bridge_if_not_exist(handle: Handle, name: String) -> Result<(), Error> {
    let mut bridge_names = handle.link().get().match_name(name.clone()).execute();

    match bridge_names.try_next().await? {
        Some(_) => Ok(()),
        None => handle
            .link()
            .add()
            .bridge(name)
            .execute()
            .await
            .map_err(|_| Error::RequestFailed),
    }
}

async fn attach_link_to_bridge(
    handle: Handle,
    link_name: String,
    master_name: String,
) -> Result<(), Error> {
    let mut link_names = handle.link().get().match_name(link_name.clone()).execute();
    let mut master_names = handle
        .link()
        .get()
        .match_name(master_name.clone())
        .execute();

    let link_index = match link_names.try_next().await? {
        Some(link) => link.header.index,
        None => panic!(),
    };
    let master_index = match master_names.try_next().await? {
        Some(link) => link.header.index,
        None => panic!(),
    };

    let _ = handle
        .link()
        .set(link_index)
        .controller(master_index)
        .execute()
        .await
        .map_err(|_| Error::RequestFailed);

    Ok(())
}
