
#[cfg(test)]
mod tests {
    use crate::*;
    use rstest::*;

    #[fixture]
    #[once]
    fn once_fixture() -> Arguments {
        let file_path = PathBuf::from_str(r"samples\recognition\zip\").unwrap();
        // Getting path to extensions.toml (forced to use env::current_dir())
        let mut extensions_path = env::current_dir().unwrap().clone();
        extensions_path.push("Extensions.toml");
        Arguments {
            file_path,
            extensions_path,
            is_debug: true,
            is_human: false,
            only_general: false,
            ignore_general: false,
            extension_info: false,
        }
    }

    #[rstest]
    #[case::threemf("3mf")]
    #[case::one23dx("123dx")]
    #[case::aab("aab")]
    #[case::air("air")]
    #[case::apk("apk")]
    #[case::appx("appx")]
    #[case::appxbundle("appxbundle")]
    #[case::cddx("cddx")]
    #[case::docx("docx")]
    #[case::dwfx("dwfx")]
    #[case::ear("ear")]
    #[case::f3d("f3d")]
    #[case::fbx("fbz")]
    #[case::fla("fla")]
    #[case::ipa("ipa")]
    #[case::jar("jar")]
    #[case::kmz("kmz")]
    #[case::pptx("pptx")]
    #[case::scdoc("scdoc")]
    #[case::sketch("sketch")]
    #[case::usdz("usdz")]
    #[case::vsdx("vsdx")]
    #[case::vsix("vsix")]
    #[case::war("war")]
    #[case::xap("xap")]
    #[case::xlsx("xlsx")]
    #[case::xpi("xpi")]
    #[case::xps("xps")]
    fn recognition_tests(once_fixture: &Arguments, #[case] extension: String) {
        let mut file_path = once_fixture.file_path.clone();
        file_path.push(format!("{}.zip", extension));

        let buf_reader: BufReader<fs::File> = BufReader::new(fs::File::open(file_path).unwrap());

        assert_eq!(
            get_complex_zip_extension(&once_fixture, buf_reader).unwrap(),
            extension
        );
    }
}
