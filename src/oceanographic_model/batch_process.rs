use chrono::NaiveDate;
use gdal::Dataset;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;
use crate::date_gen::DateTimeGenerator;
use crate::oceanographic_model::OceanographicProcessor;

#[derive(Debug)]
pub struct BatchProcessor {
    datasets: Vec<HashMap<String, String>>,
    config: Config,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Self {
        let datasets = Self::create_period_datasets(&config);
        BatchProcessor { datasets, config }
    }

    /// Creates datasets by finding actual files that match the date patterns
    pub fn create_period_datasets(config: &Config) -> Vec<HashMap<String, String>> {
        let mut datasets = Vec::new();
        let mut missing_dates = Vec::new();

        // Use DateTimeGenerator to generate the date series
        let date_generator = DateTimeGenerator::new(config.clone());
        let dates = date_generator.generate_date_series();
        println!("Requested {} date periods: {:?}", dates.len(), dates);

        if let Some(raster_templates) = config.raster_templates() {
            for date in &dates {
                let mut rasters = HashMap::new();
                let mut missing_templates = Vec::new();

                for template in raster_templates {
                    // Find files that match this template and contain this date
                    if let Some(matching_file) = Self::find_matching_file(template, date) {
                        rasters.insert(template.name.clone(), matching_file);
                    } else {
                        missing_templates.push(&template.name);
                    }
                }

                // Check if we found all required raster files for this date
                if rasters.len() == raster_templates.len() {
                    println!(
                        "✓ Found all {} raster files for date {}",
                        rasters.len(),
                        date
                    );
                    datasets.push(rasters);
                } else {
                    println!(
                        "✗ Missing raster files for date {}: {:?}",
                        date, missing_templates
                    );
                    missing_dates.push(*date);
                }
            }
        }

        // Error if we couldn't find files for some requested dates
        if !missing_dates.is_empty() {
            panic!(
                "ERROR: Requested {} days of data, but could only find files for {} days. Missing data for dates: {:?}",
                dates.len(),
                datasets.len(),
                missing_dates
            );
        }

        println!(
            "Successfully found files for all {} requested date periods",
            datasets.len()
        );

        datasets
    }

    /// Find a file that matches the template pattern for the specified date
    /// Searches recursively within the base directory
    fn find_matching_file(
        template: &crate::config::RasterFile,
        target_date: &NaiveDate,
    ) -> Option<String> {
        // Format the date according to the template's date format
        let formatted_date = Self::format_date_for_template(target_date, &template.date_format);

        // Generate the expected filename by replacing {} with the formatted date
        let expected_filename = template.filename_pattern.replace("{}", &formatted_date);

        // First try direct path (backwards compatibility)
        let direct_path = format!("{}/{}", template.base_directory, expected_filename);
        if Path::new(&direct_path).exists() {
            return Some(direct_path);
        }

        // If not found directly, search recursively in base directory
        Self::search_file_recursively(&template.base_directory, &expected_filename)
    }

    /// Search for a file recursively within a directory
    fn search_file_recursively(base_dir: &str, filename: &str) -> Option<String> {
        if !Path::new(base_dir).exists() {
            return None;
        }

        for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file()
                && let Some(file_name) = entry.path().file_name()
                && file_name.to_string_lossy() == filename
            {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }

        None
    }

    /// Formats a date according to the specified format pattern
    fn format_date_for_template(date: &NaiveDate, format: &str) -> String {
        match format {
            "YYYYMMDD" => date.format("%Y%m%d").to_string(),
            "YYYY-MM-DD" => date.format("%Y-%m-%d").to_string(),
            "YYYY_MM_DD" => date.format("%Y_%m_%d").to_string(),
            _ => date.format("%Y%m%d").to_string(), // Default to YYYYMMDD
        }
    }

    pub fn process(&self) -> Result<Vec<Dataset>, Box<dyn std::error::Error>> {
        let mut all_pp = Vec::new();
        for raster_dataset in &self.datasets {
            let proc = OceanographicProcessor::new(raster_dataset)?;
            if let Some(bbox) = self.config.bbox() {
                all_pp.push(proc.calculate_pp_for_bbox(bbox)?);
            }
        }

        Ok(all_pp)
    }
}
