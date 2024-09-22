//! Test the example from https://github.com/chmp/serde_arrow/issues/90
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    self as serde_arrow,
    internal::error::PanicOnError,
    schema::{SchemaLike, TracingOptions},
};

use crate::_impl::arrow::{_raw::schema::Schema, array::RecordBatch, datatypes::FieldRef};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Distribution {
    pub samples: Vec<f64>,
    pub statistic: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct VectorMetric {
    pub distribution: Option<Distribution>,
}

#[test]
fn example() -> PanicOnError<()> {
    let metrics = vec![
        VectorMetric {
            distribution: Some(Distribution {
                samples: vec![1.0, 2.0, 3.0],
                statistic: String::from("metric1"),
            }),
        },
        VectorMetric {
            distribution: Some(Distribution {
                samples: vec![4.0, 5.0, 6.0],
                statistic: String::from("metric2"),
            }),
        },
        VectorMetric { distribution: None },
    ];

    let fields = Vec::<FieldRef>::from_type::<VectorMetric>(TracingOptions::default())?;
    let arrays = serde_arrow::to_arrow(&fields, &metrics)?;

    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields.clone())), arrays.clone())?;
    println!("{:#?}", batch);

    let round_tripped: Vec<VectorMetric> = serde_arrow::from_arrow(&fields, &arrays)?;
    assert_eq!(metrics, round_tripped);

    Ok(())
}

#[test]
fn example_top_level_none() -> PanicOnError<()> {
    // top-level options are not supported if fields are are extracted
    let res = Vec::<FieldRef>::from_type::<Option<Distribution>>(TracingOptions::default());
    assert!(res.is_err());
    Ok(())
}
