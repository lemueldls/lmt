// use picante::PicanteResult;

use crate::ModuleId;

#[picante::input]
pub struct NamedSource {
    #[key]
    pub name: String,
    pub content: String,
    pub module_id: ModuleId,
}

// #[picante::tracked]
// pub fn line_starts<DB: ReportDatabaseTrait>(
//     db: &DB,
//     source: NamedSource,
// ) -> PicanteResult<Vec<usize>> {
//     let content = source.content(db)?;

//     Ok(content
//         .lines()
//         .enumerate()
//         .filter_map(|(i, line)| if line.is_empty() { None } else { Some(i) })
//         .collect())
// }
