#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;

pub mod constants;
use fontspector_checkapi::{prelude::*, ProfileBuilder, Registry};

mod network_conditions;
mod utils;
use serde_json::json;
use std::collections::HashMap;

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        let desc = FileType::new("DESCRIPTION.en_us.html");
        cr.register_filetype("MDPB", mdpb);
        cr.register_filetype("DESC", desc);

        let builder = ProfileBuilder::new()
            .include_profile("universal")
            .add_section("Article Checks")
//            .add_and_register_check(checks::article::images)

            .add_section("Metadata Checks")
//            .add_and_register_check(checks::metadata::axisregistry_bounds)
//            .add_and_register_check(checks::metadata::axisregistry_valid_tags)
//            .add_and_register_check(checks::metadata::broken_links)
//            .add_and_register_check(checks::metadata::canonical_style_names)
//            .add_and_register_check(checks::metadata::canonical_weight_value)
            .add_and_register_check(checks::metadata::can_render_samples)
//            .add_and_register_check(checks::metadata::category)
//            .add_and_register_check(checks::metadata::category_hints)
//            .add_and_register_check(checks::metadata::consistent_axis_enumeration)
//            .add_and_register_check(checks::metadata::consistent_repo_urls)
            .add_and_register_check(checks::metadata::copyright)
//            .add_and_register_check(checks::metadata::date_added) // Merged into metadata/parses
//            .add_and_register_check(checks::metadata::designer_profiles)
//            .add_and_register_check(checks::metadata::designer_values) // Merged into metadata/parses
//            .add_and_register_check(checks::metadata::empty_designer)
//            .add_and_register_check(checks::metadata::escaped_strings)
//            .add_and_register_check(checks::metadata::family_directory_name)
//            .add_and_register_check(checks::metadata::familyname)
//            .add_and_register_check(checks::metadata::filenames)
//            .add_and_register_check(checks::metadata::has_regular)
//            .add_and_register_check(checks::metadata::includes_production_subsets)
            .add_and_register_check(checks::metadata::license)
//            .add_and_register_check(checks::metadata::match_filename_postscript)
//            .add_and_register_check(checks::metadata::match_fullname_postscript)
//            .add_and_register_check(checks::metadata::match_name_familyname)
//            .add_and_register_check(checks::metadata::match_weight_postscript)
//            .add_and_register_check(checks::metadata::menu_and_latin) // Merged into subsets/correct
//            .add_and_register_check(checks::metadata::minisite_url)
//            .add_and_register_check(checks::metadata::nameid/family_and_full_names)
//            .add_and_register_check(checks::metadata::nameid/font_name)
//            .add_and_register_check(checks::metadata::nameid/post_script_name)
//            .add_and_register_check(checks::metadata::parses)
//            .add_and_register_check(checks::metadata::primary_script)
//            .add_and_register_check(checks::metadata::regular_is_400)
//            .add_and_register_check(checks::metadata::single_cjk_subset) // To merge into metadata/subsets_correct
            .add_and_register_check(checks::metadata::subsets_correct)  // Replacement for metadata/unsupported_subsets
//            .add_and_register_check(checks::metadata::subsets_order)  // Merged into metadata/validate
//            .add_and_register_check(checks::metadata::undeclared_fonts)
//            .add_and_register_check(checks::metadata::unique_full_name_values)
//            .add_and_register_check(checks::metadata::unique_weight_style_pairs)
//            .add_and_register_check(checks::metadata::unreachable_subsetting)
            .add_and_register_check(checks::metadata::validate)
//            .add_and_register_check(checks::metadata::valid_filename_values)
//            .add_and_register_check(checks::metadata::valid_full_name_values)
//            .add_and_register_check(checks::metadata::valid_nameid25)
//            .add_and_register_check(checks::metadata::valid_post_script_name_values)
//            .add_and_register_check(checks::metadata::weightclass)

            .add_section("Glyphset Checks")
//            .add_and_register_check(checks::glyphsets::shape_languages)
            .add_and_register_check(checks::tofu)

            .add_section("Description Checks")
//            .add_and_register_check(checks::description::broken_links)
            .add_and_register_check(checks::description::eof_linebreak)
//            .add_and_register_check(checks::description::family_update)
//            .add_and_register_check(checks::description::git_url)
//            .add_and_register_check(checks::description::has_article)
//            .add_and_register_check(checks::description::has_unsupported_elements)
            .add_and_register_check(checks::description::min_length)
//            .add_and_register_check(checks::description::urls)
//            .add_and_register_check(checks::description::valid_html)

            .add_section("Family Checks")
            .add_and_register_check(checks::family::equal_codepoint_coverage)
//            .add_and_register_check(checks::family::italics_have_roman_counterparts)
//            .add_and_register_check(checks::family::tnum_horizontal_metrics)

            .add_section("Name table checks")
//            .add_and_register_check(checks::name::family_name_compliance)
//            .add_and_register_check(checks::name::line_breaks)

            .add_section("Licensing Checks")
//            .add_and_register_check(checks::family::has_license)
//            .add_and_register_check(checks::font_copyright)
//            .add_and_register_check(checks::license::OFL_body_text)
//            .add_and_register_check(checks::license::OFL_copyright)
//            .add_and_register_check(checks::metadata::copyright)
//            .add_and_register_check(checks::metadata::license)
//            .add_and_register_check(checks::metadata::reserved_font_name)
//            .add_and_register_check(checks::name::license)
//            .add_and_register_check(checks::name::license_url)
            .add_and_register_check(checks::name::rfn)

            .add_section("Repository Checks")
//            .add_and_register_check(checks::repo::dirname_matches_nameid_1)
//            .add_and_register_check(checks::repo::fb_report)
//            .add_and_register_check(checks::repo::sample_image)
//            .add_and_register_check(checks::repo::upstream_yaml_has_required_fields)
//            .add_and_register_check(checks::repo::vf_has_static_fonts)
//            .add_and_register_check(checks::repo::zip_files",

            .add_section("Shaping Checks")
//            .add_and_register_check(checks::dotted_circle)
//            .add_and_register_check(checks::shaping::collides)
//            .add_and_register_check(checks::shaping::forbidden)
//            .add_and_register_check(checks::shaping::regression)
//            .add_and_register_check(checks::soft_dotted)

            .add_section("Outline Checks")
            .add_and_register_check(checks::outline::alignment_miss)
            .add_and_register_check(checks::outline::colinear_vectors)
            .add_and_register_check(checks::outline::direction)
            .add_and_register_check(checks::outline::jaggy_segments)
            .add_and_register_check(checks::outline::overlapping_path_segments)
            .add_and_register_check(checks::outline::semi_vertical)
            .add_and_register_check(checks::outline::short_segments)

            .add_section("Font File Checks")
            .add_and_register_check(checks::axes_match)
//            .add_and_register_check(checks::axisregistry::fvar_axis_defaults)
//            .add_and_register_check(checks::canonical_filename)
//            .add_and_register_check(checks::cjk_vertical_metrics)
//            .add_and_register_check(checks::cjk_vertical_metrics_regressions)
            .add_and_register_check(checks::color_fonts)
//            .add_and_register_check(checks::font_names)
            .add_and_register_check(checks::fstype)
            .add_and_register_check(checks::fvar_instances)
//            .add_and_register_check(checks::gasp)
//            .add_and_register_check(checks::glyph_coverage)
//            .add_and_register_check(checks::has_ttfautohint_params)
//            .add_and_register_check(checks::meta::script_lang_tags)
            .add_and_register_check(checks::name::description_max_length)
//            .add_and_register_check(checks::name::familyname_first_char)
//            .add_and_register_check(checks::name::mandatory_entries)
//            .add_and_register_check(checks::name::version_format)
//            .add_and_register_check(checks::old_ttfautohint)
//            .add_and_register_check(checks::production_encoded_glyphs)  // DISABLED
//            .add_and_register_check(checks::production_glyphs_similarity)
            .add_and_register_check(checks::render_own_name)
//            .add_and_register_check(checks::STAT)
//            .add_and_register_check(checks::STAT::axis_order)
//            .add_and_register_check(checks::STAT::axisregistry)
//            .add_and_register_check(checks::unitsperem)
            .add_and_register_check(checks::use_typo_metrics)
//            .add_and_register_check(checks::varfont::generate_static)
//            .add_and_register_check(checks::varfont::has_HVAR)
//            .add_and_register_check(checks::vendor_id)
//            .add_and_register_check(checks::version_bump)
//            .add_and_register_check(checks::vertical_metrics)
//            .add_and_register_check(checks::vertical_metrics_regressions)
            .add_and_register_check(checks::weightclass)

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
