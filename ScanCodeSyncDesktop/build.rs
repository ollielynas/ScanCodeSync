fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/favicon.ico");
        res.compile().unwrap();
    }
}
