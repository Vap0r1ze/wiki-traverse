use indicatif::{ProgressBar, ProgressStyle};

pub fn create(size: u64, bytes: bool) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::with_template(&format!(
            "{{spinner:.green}} [{{elapsed_precise}}] {{msg}} [{{wide_bar:.cyan/blue}}] {} ({{eta}})",
            if bytes {
                "{bytes}/{total_bytes}"
            } else {
                "{pos}/{len}"
            }
        ))
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}
