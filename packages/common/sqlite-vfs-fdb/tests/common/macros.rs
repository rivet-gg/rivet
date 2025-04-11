// No need to import here since this is just defining a macro
// The imports will be handled by the file using the macro

// Define a macro for creating a test that will export metrics at the end
#[macro_export]
macro_rules! define_test_with_metrics {
    (
        $name:ident, 
        $body:expr,
        $test_prefix:expr
    ) => {
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            // Run the actual test
            let result = $body();
            
            // Export metrics regardless of test result
            {
                use prometheus::{self, TextEncoder, Encoder};
                use std::fs::File;
                use std::io::Write;
                use std::path::PathBuf;
                
                // Gather and encode metrics
                let mut buffer = Vec::new();
                let encoder = TextEncoder::new();
                let metric_families = prometheus::gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                let metrics_output = String::from_utf8(buffer).unwrap();
                
                // Create metrics file in target directory
                let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                file_path.push("target");
                std::fs::create_dir_all(&file_path).unwrap();
                file_path.push(format!("sqlite_vfs_fdb_metrics_{}_{}.txt", $test_prefix, stringify!($name)));
                
                // Write metrics to file
                let mut file = File::create(&file_path).unwrap();
                file.write_all(metrics_output.as_bytes()).unwrap();
                
                // Print the file path
                println!("Metrics for {} exported to: {}", stringify!($name), file_path.display());
            }
            
            // Return the test result
            result
        }
    };
}

