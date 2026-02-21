fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "abitti-yo.yphandler.exe");
    res.set("ProductName", "Abitti Yritysportaali Updater");
    res.set("CompanyName", "Gradia");
    res.set("LegalCopyright", "Copyright Gradia © 2026");
    res.compile().unwrap();
}