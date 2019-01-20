//! Module with the definition of fn's and struct's to read .dbf files

use std::io::{Read};
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

use byteorder::{ReadBytesExt};

use header::Header;
use record::{RecordFieldInfo};
use record::field::FieldValue;
use Error;

pub type Record = HashMap<String, FieldValue>;

/// Struct with the handle to the source .dbf file
/// Responsible for reading the content
pub struct Reader<T: Read> {
    source: T,
    header: Header,
    fields_info: Vec<RecordFieldInfo>,
    current_record: u32,
}

impl<T: Read> Reader<T> {
    /// Creates a new reader from the source.
    ///
    /// Reads the header and fields information as soon as its created.
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs::File;
    /// let f = File::open("tests/data/line.dbf").unwrap();
    /// let reader = dbase::Reader::new(f).unwrap();
    /// ```
    pub fn new(mut source: T) -> Result<Self, Error> {
        let header = Header::read_from(&mut source)?;
        let num_fields = (header.offset_to_first_record as usize - Header::SIZE) / RecordFieldInfo::SIZE;

        let mut fields_info = Vec::<RecordFieldInfo>::with_capacity(num_fields as usize + 1);
        fields_info.push(RecordFieldInfo::new_deletion_flag());
        for _ in 0..num_fields {
            let info = RecordFieldInfo::read_from(&mut source)?;
            //println!("{} -> {}, {:?}, length: {}", i, info.name, info.field_type, info.record_length);
            fields_info.push(info);
        }

        let terminator = source.read_u8()? as char;
        if terminator != '\r' {
            panic!("unexpected terminator");
        }

        Ok(Self {
            source,
            header,
            fields_info,
            current_record: 0,
        })
    }

    /// Make the `Reader` read the [Records](type.Record.html)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    ///
    /// let f = File::open("tests/data/line.dbf").unwrap();
    /// let reader = dbase::Reader::new(f).unwrap();
    /// let records = reader.read().unwrap();
    /// assert_eq!(records.len(), 1);
    /// ```
    pub fn read(self) -> Result<Vec<Record>, Error> {
        let mut records = Vec::<Record>::with_capacity(self.fields_info.len());
        for record in self {
            records.push(record?);
        }
        //let file_end = self.source.read_u16::<LittleEndian>()?;
        Ok(records)
    }
}


impl<T: Read> Iterator for Reader<T> {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.current_record >= self.header.num_records {
            None
        } else {
            let mut record = Record::with_capacity(self.fields_info.len() as usize);
            for field_info in &self.fields_info {
                let value = match FieldValue::read_from(&mut self.source, field_info) {
                    Err(e) => return Some(Err(e)),
                    Ok(value) => value,
                };

                if field_info.name != "DeletionFlag" {
                    record.insert(field_info.name.clone(), value);
                }
            }
            self.current_record += 1;
            Some(Ok(record))
        }
    }
}

/// One liner to read the content of a .dbf file
///
/// # Example
///
/// ```
/// let records = dbase::read("tests/data/line.dbf").unwrap();
/// assert_eq!(records.len(), 1);
/// ```
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Error> {
    let f = File::open(path)?;
    let reader = Reader::new(f)?;
    reader.read()
}