use std::path::Path;

use thiserror::Error;

use crate::{
    database::{DbError, GeneratorDatabase},
    import::{HskImport, ImportError, insert_words, parse_hsk},
    inputs::{InputError, VerifiedInputs, verify},
};

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error(transparent)]
    Inputs(#[from] InputError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Import(#[from] ImportError),
}

#[derive(Debug)]
pub struct PreparedGeneration {
    inputs: VerifiedInputs,
    database: GeneratorDatabase,
}

impl PreparedGeneration {
    pub fn inputs(&self) -> &VerifiedInputs {
        &self.inputs
    }

    pub fn database(&self) -> &GeneratorDatabase {
        &self.database
    }
}

pub fn prepare(manifest_path: &Path) -> Result<PreparedGeneration, GeneratorError> {
    let inputs = verify(manifest_path)?;
    let database = GeneratorDatabase::open()?;
    crate::schema::create_schema(database.connection())?;
    Ok(PreparedGeneration { inputs, database })
}

pub fn import_primary(prepared: &PreparedGeneration) -> Result<HskImport, GeneratorError> {
    let source = prepared
        .inputs
        .sources()
        .iter()
        .find(|source| source.name() == "hsk-primary")
        .ok_or(ImportError::MissingSource {
            name: "hsk-primary",
        })?;
    let imported = parse_hsk(source.file().bytes())?;
    insert_words(prepared.database.connection(), &imported.words)?;
    Ok(imported)
}
