use polars::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

pub struct LazyFramer {
    pub lazyframe: Option<LazyFrame>,
    pub columns: Vec<String>,
}

impl LazyFramer {
    pub fn new(files: Vec<PathBuf>) -> Self {
        let files_arc: Arc<[PathBuf]> = Arc::from(files);
        let args = ScanArgsParquet::default();
        log::info!("Files {:?}", files_arc);

        match LazyFrame::scan_parquet_files(files_arc, args) {
            Ok(lf) => {
                log::info!("Loaded Parquet files");
                let column_names = Self::get_column_names_from_lazyframe(&lf);

                Self {
                    lazyframe: Some(lf),
                    columns: column_names, 
                }
            }
            Err(e) => {
                log::error!("Failed to load Parquet files: {}", e);
                Self {
                    lazyframe: None, // Indicates that loading failed
                    columns: Vec::new(),
                }
            }
        }
    }

    pub fn get_column_names(&self) -> Vec<String> {
        self.columns.clone()
    }

    // Adjusted signature to match context
    pub fn get_column_names_from_lazyframe(lazyframe: &LazyFrame) -> Vec<String> {
        let lf: LazyFrame = lazyframe.clone().limit(1);
        let df: DataFrame = lf.collect().unwrap(); 
        let columns: Vec<String> = df.get_column_names_owned().into_iter().map(|name| name.to_string()).collect();

        columns 
    }

}

    // pub fn new(lazyframe: LazyFrame) -> Self {

    //     // have to collect lazyframe to get the column names
    //     // only take the first row to minimize work
    //     // Assuming `lf` is your LazyFrame

    //     let lf: LazyFrame = lazyframe.clone().limit(1); // Limit the LazyFrame to just the first row
    //     // Now, collect the limited LazyFrame
    //     let df: DataFrame = lf.collect().unwrap(); // Handle this Result appropriately in real code

    //     let columns: Vec<String> = df.get_column_names_owned().into_iter().map(|name| name.to_string()).collect();
    //     Self {
    //         lazyframe,
    //         columns,
    //     }
    // }