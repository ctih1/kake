fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("FileDescription", "abitti_account_mgr.exe");
    res.set("ProductName", "Abitti Windows Account Manager");
    res.set("CompanyName", "Ylioppilastutkintolautakunta");
    res.set("LegalCopyright", "Copyright Abitti © 2026");
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
    res.set_language(0x0009);
    
    res.compile().unwrap();
}