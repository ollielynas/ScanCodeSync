#[macro_export]
macro_rules! populate_field {
    ($state:expr, $field:ident, $value:expr) => {
        $state.$field.populate(Box::new($value), stringify!($field))
    };
}
#[macro_export]
macro_rules! load_field {
    ($state:expr, $field:ident) => {
        // Automatically calls load_from_file("input_folder") for state.input_folder
        $state.$field.load_from_file(stringify!($field))
    };
}

 #[macro_export]
macro_rules! depopulate_all_into_state_init_values {
    ($state:expr, $id:expr, [$($field:ident),*]) => {
        { // Added a block here for safety
            if ![$($state.$field.available()),*].iter().all(|&x| x) {
                anyhow::bail!("Not all values are available for setup");
            }
            StateInitValues {
                $($field: $state.$field.depopulate($id)?),*
            }
        } // No semicolon here!
    };
}

#[macro_export]
macro_rules! dbp {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) { println!($($arg)*) }
    };
}
