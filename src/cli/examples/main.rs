fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut prev = 0;
            let mut curr = 1;
            for _ in 2..=n {
                let next = prev + curr;
                prev = curr;
                curr = next;
            }
            curr
        }
    }
}

fn main() {
    let n = 10;

    for i in 0..=10 {
        println!("🚀 Loop {}", i);
        println!("Fibonacci number at position {} is: {}", n, fibonacci(n));
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
