use std::fs;

fn main() {
    // 将 assets 目录下的所有文件复制到构建目录
    fs::create_dir_all("target/release/assets").expect("Failed to create assets directory");

    let files = fs::read_dir("assets").expect("Failed to read assets directory");
    for file in files {
        let file = file.expect("Failed to read file entry");
        let dest = format!("target/release/assets/{}", file.file_name().to_string_lossy());
        fs::copy(file.path(), dest).expect("Failed to copy file");
    }
}
