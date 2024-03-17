use crate::{
    _impl::arrow2::datatypes::{DataType, Field, IntegerType, TimeUnit, UnionMode},
    internal::{
        error::{error, fail, Error, Result},
        schema::{
            GenericDataType, GenericField, GenericTimeUnit, SchemaLike, Sealed, SerdeArrowSchema,
            Strategy, STRATEGY_KEY,
        },
    },
};

/// Support for arrow2 types (*requires one of the `arrow2-*` features*)
impl SerdeArrowSchema {
    /// Build a new Schema object from fields
    pub fn from_arrow2_fields(fields: &[Field]) -> Result<Self> {
        Ok(Self {
            fields: fields
                .iter()
                .map(GenericField::try_from)
                .collect::<Result<_>>()?,
        })
    }

    /// This method is deprecated. Use
    /// [`to_arrow2_fields`][SerdeArrowSchema::to_arrow2_fields] instead:
    ///
    /// ```rust
    /// # fn main() -> serde_arrow::_impl::PanicOnError<()> {
    /// # use serde_arrow::schema::{SerdeArrowSchema, SchemaLike, TracingOptions};
    /// # #[derive(serde::Deserialize)]
    /// # struct Item { a: u32 }
    /// # let schema = SerdeArrowSchema::from_type::<Item>(TracingOptions::default()).unwrap();
    /// # let fields =
    /// schema.to_arrow2_fields()?
    /// # ;
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated = "The method `get_arrow2_fields` is deprecated. Use `to_arrow2_fields` instead"]
    pub fn get_arrow2_fields(&self) -> Result<Vec<Field>> {
        self.to_arrow2_fields()
    }

    /// Build a vec of fields from a  Schema object
    pub fn to_arrow2_fields(&self) -> Result<Vec<Field>> {
        self.fields.iter().map(Field::try_from).collect()
    }
}

impl TryFrom<SerdeArrowSchema> for Vec<Field> {
    type Error = Error;

    fn try_from(value: SerdeArrowSchema) -> Result<Self> {
        value.to_arrow2_fields()
    }
}

impl Sealed for Vec<Field> {}

/// Schema support for `Vec<arrow2::datatype::Field>` (*requires one of the
/// `arrow2-*` features*)
impl SchemaLike for Vec<Field> {
    fn from_value<T: serde::Serialize + ?Sized>(value: &T) -> Result<Self> {
        SerdeArrowSchema::from_value(value)?.to_arrow2_fields()
    }

    fn from_type<'de, T: serde::Deserialize<'de> + ?Sized>(
        options: crate::schema::TracingOptions,
    ) -> Result<Self> {
        SerdeArrowSchema::from_type::<T>(options)?.to_arrow2_fields()
    }

    fn from_samples<T: serde::Serialize + ?Sized>(
        samples: &T,
        options: crate::schema::TracingOptions,
    ) -> Result<Self> {
        SerdeArrowSchema::from_samples(samples, options)?.to_arrow2_fields()
    }
}

impl TryFrom<&Field> for GenericField {
    type Error = Error;

    fn try_from(field: &Field) -> Result<Self> {
        let strategy: Option<Strategy> = match field.metadata.get(STRATEGY_KEY) {
            Some(strategy_str) => Some(strategy_str.parse::<Strategy>()?),
            None => None,
        };
        let name = field.name.to_owned();
        let nullable = field.is_nullable;

        let mut children = Vec::<GenericField>::new();
        let data_type = match &field.data_type {
            DataType::Boolean => GenericDataType::Bool,
            DataType::Null => GenericDataType::Null,
            DataType::Int8 => GenericDataType::I8,
            DataType::Int16 => GenericDataType::I16,
            DataType::Int32 => GenericDataType::I32,
            DataType::Int64 => GenericDataType::I64,
            DataType::UInt8 => GenericDataType::U8,
            DataType::UInt16 => GenericDataType::U16,
            DataType::UInt32 => GenericDataType::U32,
            DataType::UInt64 => GenericDataType::U64,
            DataType::Float16 => GenericDataType::F16,
            DataType::Float32 => GenericDataType::F32,
            DataType::Float64 => GenericDataType::F64,
            DataType::Utf8 => GenericDataType::Utf8,
            DataType::LargeUtf8 => GenericDataType::LargeUtf8,
            DataType::Date32 => GenericDataType::Date32,
            DataType::Date64 => GenericDataType::Date64,
            DataType::Decimal(precision, scale) => {
                if *precision > u8::MAX as usize || *scale > i8::MAX as usize {
                    fail!("cannot represent precision / scale of the decimal");
                }
                GenericDataType::Decimal128(*precision as u8, *scale as i8)
            }
            DataType::Time64(TimeUnit::Microsecond) => {
                GenericDataType::Time64(GenericTimeUnit::Microsecond)
            }
            DataType::Time64(TimeUnit::Nanosecond) => {
                GenericDataType::Time64(GenericTimeUnit::Nanosecond)
            }
            DataType::Time64(unit) => fail!("Invalid time unit {unit:?} for Time64"),
            DataType::Timestamp(TimeUnit::Second, tz) => {
                GenericDataType::Timestamp(GenericTimeUnit::Second, tz.clone())
            }
            DataType::Timestamp(TimeUnit::Millisecond, tz) => {
                GenericDataType::Timestamp(GenericTimeUnit::Millisecond, tz.clone())
            }
            DataType::Timestamp(TimeUnit::Microsecond, tz) => {
                GenericDataType::Timestamp(GenericTimeUnit::Microsecond, tz.clone())
            }
            DataType::Timestamp(TimeUnit::Nanosecond, tz) => {
                GenericDataType::Timestamp(GenericTimeUnit::Nanosecond, tz.clone())
            }
            DataType::List(field) => {
                children.push(GenericField::try_from(field.as_ref())?);
                GenericDataType::List
            }
            DataType::LargeList(field) => {
                children.push(field.as_ref().try_into()?);
                GenericDataType::LargeList
            }
            DataType::Struct(fields) => {
                for field in fields {
                    children.push(field.try_into()?);
                }
                GenericDataType::Struct
            }
            DataType::Map(field, _) => {
                children.push(field.as_ref().try_into()?);
                GenericDataType::Map
            }
            DataType::Union(fields, field_indices, mode) => {
                if field_indices.is_some() {
                    fail!("Union types with explicit field indices are not supported");
                }
                if !mode.is_dense() {
                    fail!("Only dense unions are supported at the moment");
                }

                for field in fields {
                    children.push(field.try_into()?);
                }
                GenericDataType::Union
            }
            DataType::Dictionary(int_type, data_type, sorted) => {
                if *sorted {
                    fail!("Sorted dictionary are not supported");
                }
                let key_type = match int_type {
                    IntegerType::Int8 => DataType::Int8,
                    IntegerType::Int16 => DataType::Int16,
                    IntegerType::Int32 => DataType::Int32,
                    IntegerType::Int64 => DataType::Int64,
                    IntegerType::UInt8 => DataType::UInt8,
                    IntegerType::UInt16 => DataType::UInt16,
                    IntegerType::UInt32 => DataType::UInt32,
                    IntegerType::UInt64 => DataType::UInt64,
                };
                children.push((&Field::new("", key_type, false)).try_into()?);
                children.push((&Field::new("", data_type.as_ref().clone(), false)).try_into()?);
                GenericDataType::Dictionary
            }
            dt => fail!("Cannot convert data type {dt:?}"),
        };

        let field = GenericField {
            data_type,
            name,
            strategy,
            children,
            nullable,
        };
        field.validate()?;

        Ok(field)
    }
}

impl TryFrom<&GenericField> for Field {
    type Error = Error;

    fn try_from(value: &GenericField) -> Result<Self> {
        let data_type = match &value.data_type {
            GenericDataType::Null => DataType::Null,
            GenericDataType::Bool => DataType::Boolean,
            GenericDataType::I8 => DataType::Int8,
            GenericDataType::I16 => DataType::Int16,
            GenericDataType::I32 => DataType::Int32,
            GenericDataType::I64 => DataType::Int64,
            GenericDataType::U8 => DataType::UInt8,
            GenericDataType::U16 => DataType::UInt16,
            GenericDataType::U32 => DataType::UInt32,
            GenericDataType::U64 => DataType::UInt64,
            GenericDataType::F16 => DataType::Float16,
            GenericDataType::F32 => DataType::Float32,
            GenericDataType::F64 => DataType::Float64,
            GenericDataType::Date32 => DataType::Date32,
            GenericDataType::Date64 => DataType::Date64,
            GenericDataType::Time64(GenericTimeUnit::Microsecond) => {
                DataType::Time64(TimeUnit::Microsecond)
            }
            GenericDataType::Time64(GenericTimeUnit::Nanosecond) => {
                DataType::Time64(TimeUnit::Nanosecond)
            }
            GenericDataType::Time64(unit) => fail!("Invalid time unit {unit} for Time64"),
            GenericDataType::Timestamp(GenericTimeUnit::Second, tz) => {
                DataType::Timestamp(TimeUnit::Second, tz.clone())
            }
            GenericDataType::Timestamp(GenericTimeUnit::Millisecond, tz) => {
                DataType::Timestamp(TimeUnit::Millisecond, tz.clone())
            }
            GenericDataType::Timestamp(GenericTimeUnit::Microsecond, tz) => {
                DataType::Timestamp(TimeUnit::Microsecond, tz.clone())
            }
            GenericDataType::Timestamp(GenericTimeUnit::Nanosecond, tz) => {
                DataType::Timestamp(TimeUnit::Nanosecond, tz.clone())
            }
            GenericDataType::Decimal128(precision, scale) => {
                if *scale < 0 {
                    fail!("arrow2 does not support decimals with negative scale");
                }
                DataType::Decimal(*precision as usize, *scale as usize)
            }
            GenericDataType::Utf8 => DataType::Utf8,
            GenericDataType::LargeUtf8 => DataType::LargeUtf8,
            GenericDataType::List => DataType::List(Box::new(
                value
                    .children
                    .first()
                    .ok_or_else(|| error!("List must a single child"))?
                    .try_into()?,
            )),
            GenericDataType::LargeList => DataType::LargeList(Box::new(
                value
                    .children
                    .first()
                    .ok_or_else(|| error!("List must a single child"))?
                    .try_into()?,
            )),
            GenericDataType::Struct => DataType::Struct(
                value
                    .children
                    .iter()
                    .map(Field::try_from)
                    .collect::<Result<Vec<_>>>()?,
            ),
            GenericDataType::Map => {
                let element_field: Field = value
                    .children
                    .first()
                    .ok_or_else(|| error!("Map must a two children"))?
                    .try_into()?;
                DataType::Map(Box::new(element_field), false)
            }
            GenericDataType::Union => DataType::Union(
                value
                    .children
                    .iter()
                    .map(Field::try_from)
                    .collect::<Result<Vec<_>>>()?,
                None,
                UnionMode::Dense,
            ),
            GenericDataType::Dictionary => {
                let Some(key_field) = value.children.first() else {
                    fail!("Dictionary must a two children");
                };
                let val_field: Field = value
                    .children
                    .get(1)
                    .ok_or_else(|| error!("Dictionary must a two children"))?
                    .try_into()?;

                let key_type = match &key_field.data_type {
                    GenericDataType::U8 => IntegerType::UInt8,
                    GenericDataType::U16 => IntegerType::UInt16,
                    GenericDataType::U32 => IntegerType::UInt32,
                    GenericDataType::U64 => IntegerType::UInt64,
                    GenericDataType::I8 => IntegerType::Int8,
                    GenericDataType::I16 => IntegerType::Int16,
                    GenericDataType::I32 => IntegerType::Int32,
                    GenericDataType::I64 => IntegerType::Int64,
                    _ => fail!("Invalid key type for dictionary"),
                };

                DataType::Dictionary(key_type, Box::new(val_field.data_type), false)
            }
        };

        let mut field = Field::new(&value.name, data_type, value.nullable);
        if let Some(strategy) = value.strategy.as_ref() {
            field.metadata = strategy.clone().into();
        }

        Ok(field)
    }
}
