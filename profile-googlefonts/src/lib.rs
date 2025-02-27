#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;

pub mod constants;
use fontspector_checkapi::{prelude::*, ProfileBuilder, Registry};

mod network_conditions;
mod utils;
use serde_json::json;
use std::collections::HashMap;

pub(crate) const LICENSE: FileType = FileType {
    pattern: "{LICENSE,OFL}.txt",
};

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        let desc = FileType::new("*.en_us.html");
        cr.register_filetype("MDPB", mdpb);
        cr.register_filetype("DESC", desc);
        cr.register_filetype("LICENSE", LICENSE);

        let builder = ProfileBuilder::new()
            .include_profile("universal")
            //        pending_review:
            //            checks::cmap::format_12
            //            checks::empty_letters
            //            checks::inconsistencies_between_fvar_STAT
            //            checks::no_mac_entries
            //            checks::typographic_family_name
            //            checks::vtt_volt_data
            .add_section("Article Checks")
            //            .add_and_register_check(checks::googlefonts::article::images)
            .add_section("Metadata Checks")
            .add_and_register_check(checks::googlefonts::metadata::axes);
        //            checks::googlefonts::metadata::axisregistry_bounds // Merged into metadata/axes
        //            checks::googlefonts::metadata::axisregistry_valid_tags // Merged into metadata/axes
        //            checks::googlefonts::metadata::consistent_axis_enumeration // Merged into metadata/axes
        #[cfg(not(target_family = "wasm"))]
        let builder = builder.add_and_register_check(checks::googlefonts::metadata::broken_links);
        let builder = builder
            //            checks::googlefonts::metadata::canonical_weight_value // Merged into metadata/validate
            //            checks::googlefonts::metadata::designer_values // Merged into metadata/validate
            //            checks::googlefonts::metadata::empty_designer // Merged into metadata/validate
            .add_and_register_check(checks::googlefonts::metadata::can_render_samples)
            .add_and_register_check(checks::googlefonts::metadata::category)
            //            checks::googlefonts::metadata::category_hints // merged into metadata/validate
            .add_and_register_check(checks::googlefonts::metadata::consistent_repo_urls)
            .add_and_register_check(checks::googlefonts::metadata::consistent_with_fonts)
            //            checks::googlefonts::metadata::filenames // merged into metadata/consistent_with_fonts
            //            checks::googlefonts::metadata::canonical_style_names // merged into metadata/consistent_with_fonts
            //            checks::googlefonts::metadata::valid_full_name_values // merged into metadata/consistent_with_fonts
            //            checks::googlefonts::metadata::nameid/post_script_name // merged into metadata/consistent_with_fonts
            //            checks::googlefonts::metadata::valid_post_script_name_values // merged into metadata/consistent_with_fonts
            //            checks::googlefonts::metadata::valid_filename_values // redundant, see fontbakery#4997
            //            checks::googlefonts::metadata::undeclared_fonts // redundant, see fontbakery#4997
            //            checks::googlefonts::metadata::nameid/font_name // redundant, see fontbakery#4581
            //            .add_and_register_check(checks::googlefonts::metadata::designer_profiles)
            .add_and_register_check(checks::googlefonts::metadata::escaped_strings)
            //            .add_and_register_check(checks::googlefonts::metadata::family_directory_name)
            .add_and_register_check(checks::googlefonts::metadata::familyname)
            .add_and_register_check(checks::googlefonts::metadata::has_regular)
            //            checks::googlefonts::metadata::match_filename_postscript // Merged into metadata/validate
            //            checks::googlefonts::metadata::match_fullname_postscript // Merged into metadata/validate
            //            checks::googlefonts::metadata::match_name_familyname // Merged into metadata/validate
            //            checks::googlefonts::metadata::match_weight_postscript // Merged into metadata/validate
            //            checks::googlefonts::metadata::minisite_url // Merged into metadata/validate
            //            .add_and_register_check(checks::googlefonts::metadata::nameid/family_and_full_names)
            //            .add_and_register_check(checks::googlefonts::metadata::primary_script)
            .add_and_register_check(checks::googlefonts::metadata::regular_is_400)
            //            .add_and_register_check(checks::googlefonts::metadata::single_cjk_subset) // To merge into metadata/subsets_correct
            .add_and_register_check(checks::googlefonts::metadata::subsets_correct) // Replacement for metadata/unsupported_subsets
            //            .add_and_register_check(checks::googlefonts::metadata::unique_full_name_values)
            //            .add_and_register_check(checks::googlefonts::metadata::unique_weight_style_pairs)
            //            .add_and_register_check(checks::googlefonts::metadata::unreachable_subsetting)
            .add_and_register_check(checks::googlefonts::metadata::validate)
            .add_and_register_check(checks::googlefonts::metadata::valid_nameid25)
            //            .add_and_register_check(checks::googlefonts::metadata::weightclass)
            .add_section("Glyphset Checks")
            //            .add_and_register_check(checks::googlefonts::glyphsets::shape_languages)
            .add_and_register_check(checks::googlefonts::tofu)
            .add_section("Description Checks");

        #[cfg(not(target_family = "wasm"))]
        let builder =
            builder.add_and_register_check(checks::googlefonts::description::broken_links);

        let builder = builder
            .add_and_register_check(checks::googlefonts::description::eof_linebreak)
            //            .add_and_register_check(checks::googlefonts::description::family_update)
            .add_and_register_check(checks::googlefonts::description::git_url)
            .add_and_register_check(checks::googlefonts::description::has_article)
            .add_and_register_check(checks::googlefonts::description::has_unsupported_elements)
            .add_and_register_check(checks::googlefonts::description::min_length)
            .add_and_register_check(checks::googlefonts::description::urls)
            .add_and_register_check(checks::googlefonts::description::valid_html)
            .add_section("Family Checks")
            .add_and_register_check(checks::googlefonts::family::equal_codepoint_coverage)
            //            .add_and_register_check(checks::googlefonts::family::italics_have_roman_counterparts)
            //            .add_and_register_check(checks::googlefonts::family::tnum_horizontal_metrics)
            .add_section("Name table checks")
            .add_and_register_check(checks::googlefonts::name::family_name_compliance)
            .add_and_register_check(checks::googlefonts::name::line_breaks)
            .add_section("Licensing Checks")
            .add_and_register_check(checks::googlefonts::family::has_license)
            .add_and_register_check(checks::googlefonts::font_copyright)
            .add_and_register_check(checks::googlefonts::license::OFL_body_text)
            .add_and_register_check(checks::googlefonts::license::OFL_copyright)
            .add_and_register_check(checks::googlefonts::metadata::copyright)
            .add_and_register_check(checks::googlefonts::metadata::license)
            .add_and_register_check(checks::googlefonts::metadata::reserved_font_name)
            .add_and_register_check(checks::googlefonts::name::license)
            .add_and_register_check(checks::googlefonts::name::license_url)
            .add_and_register_check(checks::googlefonts::name::rfn)
            .add_section("Repository Checks")
            //            .add_and_register_check(checks::googlefonts::repo::dirname_matches_nameid_1)
            //            .add_and_register_check(checks::googlefonts::repo::fb_report)
            //            .add_and_register_check(checks::googlefonts::repo::sample_image)
            //            checks::googlefonts::repo::upstream_yaml_has_required_fields // Redundant, no upstream.yaml any more
            //            .add_and_register_check(checks::googlefonts::repo::vf_has_static_fonts)
            //            .add_and_register_check(checks::googlefonts::repo::zip_files",
            .add_section("Shaping Checks")
            .add_and_register_check(checks::dotted_circle);
        #[cfg(not(target_family = "wasm"))]
        let builder = builder
            //            Realistically Simon is the only person who uses this check, and it can wait until he needs it again.
            //            checks::shaping::collides
            .add_and_register_check(checks::shaping::forbidden)
            .add_and_register_check(checks::shaping::regression)
            .add_and_register_check(checks::soft_dotted);

        let builder = builder
            .add_section("Outline Checks")
            .add_and_register_check(checks::outline::alignment_miss)
            .add_and_register_check(checks::outline::colinear_vectors)
            .add_and_register_check(checks::outline::direction)
            .add_and_register_check(checks::outline::jaggy_segments)
            .add_and_register_check(checks::outline::overlapping_path_segments)
            .add_and_register_check(checks::outline::semi_vertical)
            .add_and_register_check(checks::outline::short_segments)
            .add_section("Font File Checks");

        #[cfg(not(target_family = "wasm"))]
        let builder = builder.add_and_register_check(checks::googlefonts::axes_match);

        let builder = builder
            .add_and_register_check(checks::googlefonts::axisregistry::fvar_axis_defaults)
            .add_and_register_check(checks::googlefonts::canonical_filename)
            //            .add_and_register_check(checks::googlefonts::cjk_vertical_metrics)
            .add_and_register_check(checks::googlefonts::color_fonts)
            .add_and_register_check(checks::googlefonts::font_names)
            .add_and_register_check(checks::googlefonts::fstype)
            .add_and_register_check(checks::googlefonts::fvar_instances)
            .add_and_register_check(checks::googlefonts::gasp)
            .add_and_register_check(checks::googlefonts::glyph_coverage)
            .add_and_register_check(checks::googlefonts::has_ttfautohint_params)
            .add_and_register_check(checks::googlefonts::meta::script_lang_tags)
            .add_and_register_check(checks::googlefonts::name::description_max_length)
            .add_and_register_check(checks::googlefonts::name::familyname_first_char)
            .add_and_register_check(checks::googlefonts::name::mandatory_entries)
            .add_and_register_check(checks::googlefonts::name::version_format)
            .add_and_register_check(checks::googlefonts::old_ttfautohint)
            // checks::googlefonts::production_encoded_glyphs // DISABLED
            // checks::googlefonts::production_glyphs_similarity // Unlikely to be useful in the short term
            .add_and_register_check(checks::googlefonts::render_own_name)
            .add_and_register_check(checks::googlefonts::STAT::axis_order)
            .add_and_register_check(checks::googlefonts::STAT::axisregistry)
            .add_and_register_check(checks::googlefonts::STAT::compulsory_axis_values)
            .add_and_register_check(checks::googlefonts::unitsperem)
            .add_and_register_check(checks::googlefonts::use_typo_metrics)
            // Not porting generate_static, see fontbakery#1727
            .add_and_register_check(checks::googlefonts::varfont::has_HVAR)
            .add_and_register_check(checks::googlefonts::vendor_id)
            .add_and_register_check(checks::googlefonts::version_bump)
            //            .add_and_register_check(checks::googlefonts::vertical_metrics)
            //            .add_and_register_check(checks::googlefonts::vertical_metrics_regressions)
            //            .add_and_register_check(checks::googlefonts::cjk_vertical_metrics_regressions)
            //            .add_and_register_check(checks::googlefonts::metadata::includes_production_subsets)
            .add_and_register_check(checks::googlefonts::weightclass)
            .with_configuration_defaults(
                "file_size",
                HashMap::from([
                    ("WARN_SIZE".to_string(), json!(1048576)), // 1Mb
                    ("FAIL_SIZE".to_string(), json!(9437184)), // 9Mb
                ]),
            );

        builder.build("googlefonts", cr)
    }
}
