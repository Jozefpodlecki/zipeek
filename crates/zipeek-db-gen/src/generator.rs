use std::path::Path;

use thiserror::Error;

use crate::{
    database::{DbError, GeneratorDatabase},
    import::{
        HskImport, ImportError, apply_cedict_fallback, expand_with_cedict, import_relationship_csv,
        insert_full_dictionary, parse_hsk,
    },
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
    let mut imported = parse_hsk(source.file().bytes())?;
    {
        let fallback = prepared
            .inputs
            .sources()
            .iter()
            .find(|source| source.name() == "cc-cedict-fallback")
            .ok_or(ImportError::MissingSource {
                name: "cc-cedict-fallback",
            })?;
        apply_cedict_fallback(&mut imported, fallback.file().bytes())?;
        let expansion = expand_with_cedict(&mut imported, fallback.file().bytes())?;
        imported.character_readings = expansion.character_readings;
        tracing::info!(
            added_words = expansion.added_words,
            "CC-CEDICT vocabulary expanded"
        );
    }
    insert_full_dictionary(
        prepared.database.connection(),
        &imported.words,
        &imported.senses,
        &imported.character_readings,
    )?;
    tracing::info!("Importing word relationships from the bundled CSV");
    import_relationship_csv(prepared.database.connection())?;
    Ok(imported)
}
