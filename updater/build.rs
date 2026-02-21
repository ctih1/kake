fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "abitti_account_mgr.exe");
    res.set("ProductName", "Abitti Account Manager");
    res.set("CompanyName", "Ylioppilastutkintolautakunta");
    res.set("LegalCopyright", "Copyright Abitti © 2026");
    res.compile().unwrap();
}