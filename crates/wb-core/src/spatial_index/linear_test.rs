//! `LinearIndex` passes the shared `SpatialIndex` suite; as the oracle it needs nothing else.

use super::LinearIndex;
use crate::spatial_index::conformance::spatial_index_conformance;

spatial_index_conformance!(LinearIndex::new());
