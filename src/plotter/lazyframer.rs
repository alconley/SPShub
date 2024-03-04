use polars::prelude::*;

pub struct LazyFramer {
    lazyframe: LazyFrame,
    columns: Vec<String>,
}

impl LazyFramer {
    pub fn new(lazyframe: LazyFrame) -> Self {
        // have to collect lazyframe to get the column names
        // only take the first row to minimize work
        // Assuming `lf` is your LazyFrame

        let lf: LazyFrame = lazyframe.clone().limit(1); // Limit the LazyFrame to just the first row
        // Now, collect the limited LazyFrame
        let df: DataFrame = lf.collect().unwrap(); // Handle this Result appropriately in real code

        let columns: Vec<String> = df.get_column_names_owned().into_iter().map(|name| name.to_string()).collect();
        Self {
            lazyframe,
            columns,
        }
    }

    pub fn get_lazyframe(&self) -> &LazyFrame {
        &self.lazyframe
    }

    pub fn get_column_names(&self) -> Vec<String> {
        self.columns.clone()
    }

    // pub fn save_lazy_frame(&self, path: &PathBuf) -> Result<(), PolarsError> {
    //     self.lazyframe.write_to_disk(path)
    // }

}