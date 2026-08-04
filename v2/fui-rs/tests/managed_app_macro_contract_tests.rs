use fui::prelude::*;

#[derive(Clone)]
struct ProjectedPage {
    shell: FlexBox,
    mounted_root: FlexBox,
}

fn build_page() -> ProjectedPage {
    let mounted_root = column();
    let shell = column();
    shell.child(&mounted_root);
    ProjectedPage {
        shell,
        mounted_root,
    }
}

fui_managed_app!(ProjectedPage, build_page, |page: &ProjectedPage| page
    .mounted_root
    .clone());

#[test]
fn managed_app_supports_custom_root_projection() {
    let page = build_page();
    let _shell = page.shell;
    let _projected_root = page.mounted_root;
}
