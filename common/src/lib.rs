#[macro_export]
macro_rules! debug {
    ($str: expr) => {
        #[cfg(debug_assertions)]
        {
            println!($str)
        }
    };
    ($str: expr, $($rest: expr),+) => {
        #[cfg(debug_assertions)]
        {
            println!($str, $($rest),+)
        }
    };
}

#[macro_export]
macro_rules! release {
    ($str: expr) => {
        #[cfg(not(debug_assertions))]
        {
            println!($str)
        }
    };
    ($str: expr, $($rest: expr),+) => {
        #[cfg(not(debug_assertions))]
        {
            println!($str, $($rest),+)
        }
    };
}
