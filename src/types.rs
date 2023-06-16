use std::collections::HashMap;

/// All data types are either primitives or nested types, which are maps, lists, or structs.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Any {
    /// A Primitive type
    Primitive(Primitive),
    /// A Struct type
    Struct(Struct),
    /// A List type.
    List(List),
    /// A Map type
    Map(Map),
}

/// Primitive Types within a schema.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Primitive {
    /// True or False
    Boolean,
    /// 32-bit signed integer, Can promote to long
    Int,
    /// 64-bit signed integer
    Long,
    /// 32-bit IEEE 753 floating bit, Can promote to double
    Float,
    /// 64-bit IEEE 753 floating bit.
    Double,
    /// Fixed point decimal
    ///
    /// - Precision can only be widened.
    /// - Scale is fixed and cannot be changed by schema evolution.
    Decimal {
        /// The number of digits in the number, precision must be 38 or less
        precision: u8,
        /// The number of digits to the right of the decimal point.
        scale: u8,
    },
    /// Calendar date without timezone or time
    Date,
    /// Time of day without date or timezone.
    ///
    /// Time values are stored with microsecond precision.
    Time,
    /// Timestamp without timezone
    ///
    /// Timestamp values are stored with microsecond precision.
    ///
    /// Timestamps without time zone represent a date and time of day regardless of zone:
    /// the time value is independent of zone adjustments (`2017-11-16 17:10:34` is always retrieved as `2017-11-16 17:10:34`).
    /// Timestamp values are stored as a long that encodes microseconds from the unix epoch.
    Timestamp,
    /// Timestamp with timezone
    ///
    /// Timestampz values are stored with microsecond precision.
    ///
    /// Timestamps with time zone represent a point in time:
    /// values are stored as UTC and do not retain a source time zone
    /// (`2017-11-16 17:10:34 PST` is stored/retrieved as `2017-11-17 01:10:34 UTC` and these values are considered identical).
    Timestampz,
    /// Arbitrary-length character sequences, Encoded with UTF-8
    ///
    /// Character strings must be stored as UTF-8 encoded byte arrays.
    String,
    /// Universally Unique Identifiers, Should use 16-byte fixed
    Uuid,
    /// Fixed-length byte array of length.
    Fixed(u64),
    /// Arbitrary-length byte array.
    Binary,
}

/// A struct is a tuple of typed values.
///
/// - Each field in the tuple is named and has an integer id that is unique in the table schema.
/// - Each field can be either optional or required, meaning that values can (or cannot) be null.
/// - Fields may be any type.
/// - Fields may have an optional comment or doc string.
/// - Fields can have default values.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Struct {
    pub fields: Vec<Field>,
}

/// A Field is the field of a struct.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    /// An integer id that is unique in the table schema
    pub id: i32,
    /// Field Name
    pub name: String,
    /// Optional or required, meaning that values can (or can not be null)
    pub required: bool,
    /// Field can have any type
    pub field_type: Any,
    /// Fields can have any optional comment or doc string.
    pub comment: Option<String>,
}

/// A list is a collection of values with some element type.
///
/// - The element field has an integer id that is unique in the table schema.
/// - Elements can be either optional or required.
/// - Element types may be any type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct List {
    /// an integer id that is unique in the table schema.
    pub element_id: i32,
    /// Optional or required, meaning that values can (or can not be null)
    pub element_required: bool,
    /// Element types may be any type.
    pub element_type: Box<Any>,
}

/// A map is a collection of key-value pairs with a key type and a value type.
///
/// - Both the key field and value field each have an integer id that is unique in the table schema.
/// - Map keys are required and map values can be either optional or required.
/// - Both map keys and map values may be any type, including nested types.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Map {
    /// an integer id that is unique in the table schema
    pub key_id: i32,
    /// Both map keys and map values may be any type, including nested types.
    pub key_type: Box<Any>,

    /// an integer id that is unique in the table schema
    pub value_id: i32,
    /// map values can be either optional or required.
    pub value_required: bool,
    /// Both map keys and map values may be any type, including nested types.
    pub value_type: Box<Any>,
}

/// A table’s schema is a list of named columns.
///
/// All data types are either primitives or nested types, which are maps, lists, or structs.
/// A table schema is also a struct type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Schema {
    /// The unique id for this schema.
    pub id: i32,
    /// A schema can optionally track the set of primitive fields that
    /// identify rows in a table, using the property identifier-field-ids
    pub identifier_field_ids: Option<Vec<i32>>,
    /// types contained in this schema.
    pub types: Struct,
}

/// Transform is used to transform predicates to partition predicates,
/// in addition to transforming data values.
///
/// Deriving partition predicates from column predicates on the table data
/// is used to separate the logical queries from physical storage: the
/// partitioning can change and the correct partition filters are always
/// derived from column predicates.
///
/// This simplifies queries because users don’t have to supply both logical
/// predicates and partition predicates.
///
/// All transforms must return `null` for a `null` input value.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Transform {
    /// Source value, unmodified
    ///
    /// - Source type could be `Any`.
    /// - Return type is the same with source type.
    Identity,
    /// Hash of value, mod `N`.
    ///
    /// Bucket partition transforms use a 32-bit hash of the source value.
    /// The 32-bit hash implementation is the 32-bit Murmur3 hash, x86
    /// variant, seeded with 0.
    ///
    /// Transforms are parameterized by a number of buckets, N. The hash mod
    /// N must produce a positive value by first discarding the sign bit of
    /// the hash value. In pseudo-code, the function is:
    ///
    /// ```text
    /// def bucket_N(x) = (murmur3_x86_32_hash(x) & Integer.MAX_VALUE) % N
    /// ```
    ///
    /// - Source type could be `int`, `long`, `decimal`, `date`, `time`,
    ///   `timestamp`, `timestamptz`, `string`, `uuid`, `fixed`, `binary`.
    /// - Return type is `int`.
    Bucket(i32),
    /// Value truncated to width `W`
    ///
    /// For `int`:
    ///
    /// - `v - (v % W)` remainders must be positive
    /// - example: W=10: 1 ￫ 0, -1 ￫ -10
    /// - note: The remainder, v % W, must be positive.
    ///
    /// For `long`:
    ///
    /// - `v - (v % W)` remainders must be positive
    /// - example: W=10: 1 ￫ 0, -1 ￫ -10
    /// - note: The remainder, v % W, must be positive.
    ///
    /// For `decimal`:
    ///
    /// - `scaled_W = decimal(W, scale(v)) v - (v % scaled_W)`
    /// - example: W=50, s=2: 10.65 ￫ 10.50
    ///
    /// For `string`:
    ///
    /// - Substring of length L: `v.substring(0, L)`
    /// - example: L=3: iceberg ￫ ice
    /// - note: Strings are truncated to a valid UTF-8 string with no more
    ///   than L code points.
    ///
    /// - Source type could be `int`, `long`, `decimal`, `string`
    /// - Return type is the same with source type.
    Truncate(i32),
    /// Extract a date or timestamp year, as years from 1970
    ///
    /// - Source type could be `date`, `timestamp`, `timestamptz`
    /// - Return type is `int`
    Year,
    /// Extract a date or timestamp month, as months from 1970-01-01
    ///
    /// - Source type could be `date`, `timestamp`, `timestamptz`
    /// - Return type is `int`
    Month,
    /// Extract a date or timestamp day, as days from 1970-01-01
    ///
    /// - Source type could be `date`, `timestamp`, `timestamptz`
    /// - Return type is `int`
    Day,
    /// Extract a timestamp hour, as hours from 1970-01-01 00:00:00
    ///
    /// - Source type could be `timestamp`, `timestamptz`
    /// - Return type is `int`
    Hour,
    /// Always produces `null`
    ///
    /// The void transform may be used to replace the transform in an
    /// existing partition field so that the field is effectively dropped in
    /// v1 tables.
    ///
    /// - Source type could be `Any`.
    /// - Return type is Source type or `int`
    Void,
}

/// Data files are stored in manifests with a tuple of partition values
/// that are used in scans to filter out files that cannot contain records
///  that match the scan’s filter predicate.
///
/// Partition values for a data file must be the same for all records stored
/// in the data file. (Manifests store data files from any partition, as long
/// as the partition spec is the same for the data files.)
///
/// Tables are configured with a partition spec that defines how to produce a tuple of partition values from a record.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PartitionSpec {
    /// The spec id.
    pub id: i32,
    /// Partition fields.
    pub fields: Vec<PartitionField>,
}

/// Field of the specified partition spec.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PartitionField {
    /// A source column id from the table’s schema
    pub source_column_id: i32,
    /// A partition field id that is used to identify a partition field
    /// and is unique within a partition spec.
    ///
    /// In v2 table metadata, it is unique across all partition specs.
    pub partition_field_id: i32,
    /// A transform that is applied to the source column to produce
    /// a partition value
    ///
    /// The source column, selected by id, must be a primitive type
    /// and cannot be contained in a map or list, but may be nested in
    /// a struct.
    pub transform: Transform,
    /// A partition name
    pub name: String,
}

/// Users can sort their data within partitions by columns to gain
/// performance. The information on how the data is sorted can be declared
/// per data or delete file, by a sort order.
///
/// - Order id `0` is reserved for the unsorted order.
/// - Sorting floating-point numbers should produce the following behavior:
///   `-NaN` < `-Infinity` < `-value` < `-0` < `0` < `value` < `Infinity`
///   < `NaN`
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SortOrder {
    /// The sort order id of this SortOrder
    pub id: i32,
    /// The order of the sort fields within the list defines the order in
    /// which the sort is applied to the data
    pub fields: Vec<SortField>,
}

/// Field of the specified sort order.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SortField {
    /// A source column id from the table’s schema
    pub source_column_id: i32,
    /// A transform that is applied to the source column to produce
    /// a partition value
    ///
    /// The source column, selected by id, must be a primitive type
    /// and cannot be contained in a map or list, but may be nested in
    /// a struct.
    pub transform: Transform,
    /// sort direction, that can only be either `asc` or `desc`
    pub direction: SortDirection,
    /// A null order that describes the order of null values when sorted.
    /// Can only be either nulls-first or nulls-last
    pub null_order: NullOrder,
}

/// sort direction, that can only be either `asc` or `desc`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortDirection {
    ASC,
    DESC,
}

/// A null order that describes the order of null values when sorted.
/// Can only be either nulls-first or nulls-last
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NullOrder {
    First,
    Last,
}

/// Snapshots are embedded in table metadata, but the list of manifests for a
/// snapshot are stored in a separate manifest list file.
///
/// A new manifest list is written for each attempt to commit a snapshot
/// because the list of manifests always changes to produce a new snapshot.
/// When a manifest list is written, the (optimistic) sequence number of the
/// snapshot is written for all new manifest files tracked by the list.
///
/// A manifest list includes summary metadata that can be used to avoid
/// scanning all of the manifests in a snapshot when planning a table scan.
/// This includes the number of added, existing, and deleted files, and a
/// summary of values for each field of the partition spec used to write the
/// manifest.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ManifestList {
    /// field: 500
    ///
    /// Location of the manifest file
    pub manifest_path: String,
    /// field: 501
    ///
    /// Length of the manifest file in bytes
    pub manifest_length: i64,
    /// field: 502
    ///
    /// ID of a partition spec used to write the manifest; must be listed
    /// in table metadata partition-specs
    pub partition_spec_id: i32,
    /// field: 517
    ///
    /// The type of files tracked by the manifest, either data or delete
    /// files; 0 for all v1 manifests
    pub content: ManifestContentType,
    /// field: 515
    ///
    /// The sequence number when the manifest was added to the table; use 0
    /// when reading v1 manifest lists
    pub sequence_number: i64,
    /// field: 516
    ///
    /// The minimum data sequence number of all live data or delete files in
    /// the manifest; use 0 when reading v1 manifest lists
    pub min_sequence_number: i64,
    /// field: 503
    ///
    /// ID of the snapshot where the manifest file was added
    pub added_snapshot_id: i64,
    /// field: 504
    ///
    /// Number of entries in the manifest that have status ADDED, when null
    /// this is assumed to be non-zero
    pub added_files_count: i32,
    /// field: 505
    ///
    /// Number of entries in the manifest that have status EXISTING (0),
    /// when null this is assumed to be non-zero
    pub existing_files_count: i32,
    /// field: 506
    ///
    /// Number of entries in the manifest that have status DELETED (2),
    /// when null this is assumed to be non-zero
    pub deleted_files_count: i32,
    /// field: 512
    ///
    /// Number of rows in all of files in the manifest that have status
    /// ADDED, when null this is assumed to be non-zero
    pub added_rows_count: i64,
    /// field: 513
    ///
    /// Number of rows in all of files in the manifest that have status
    /// EXISTING, when null this is assumed to be non-zero
    pub existing_rows_count: i64,
    /// field: 514
    ///
    /// Number of rows in all of files in the manifest that have status
    /// DELETED, when null this is assumed to be non-zero
    pub deleted_rows_count: i64,
    /// field: 507
    /// element_field: 508
    ///
    /// A list of field summaries for each partition field in the spec. Each
    /// field in the list corresponds to a field in the manifest file’s
    /// partition spec.
    pub partitions: Option<Vec<FieldSummary>>,
    /// field: 519
    ///
    /// Implementation-specific key metadata for encryption
    pub key_metadata: Option<Vec<u8>>,
}

/// Field summary for partition field in the spec.
///
/// Each field in the list corresponds to a field in the manifest file’s partition spec.
///
/// TODO: add lower_bound and upper_bound support
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FieldSummary {
    /// field: 509
    ///
    /// Whether the manifest contains at least one partition with a null
    /// value for the field
    pub contains_null: bool,
    /// field: 518
    /// Whether the manifest contains at least one partition with a NaN
    /// value for the field
    pub contains_nan: Option<bool>,
}

/// A manifest is an immutable Avro file that lists data files or delete
/// files, along with each file’s partition data tuple, metrics, and tracking
/// information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ManifestFile {
    /// field: 0
    ///
    /// Used to track additions and deletions.
    pub status: ManifestStatus,
    /// field id: 1
    ///
    /// Snapshot id where the file was added, or deleted if status is 2.
    /// Inherited when null.
    pub snapshot_id: Option<i64>,
    /// field id: 3
    ///
    /// Data sequence number of the file.
    /// Inherited when null and status is 1 (added).
    pub sequence_number: Option<i64>,
    /// field id: 4
    ///
    /// File sequence number indicating when the file was added.
    /// Inherited when null and status is 1 (added).
    pub file_sequence_number: Option<i64>,
    /// field id: 2
    ///
    /// File path, partition tuple, metrics, …
    pub data_file: DataFile,
}

/// FIXME: partition_spec is not parsed.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ManifestMetadata {
    /// The table schema at the time the manifest
    /// was written
    pub schema: Schema,
    /// ID of the schema used to write the manifest as a string
    pub schema_id: i32,

    /// The partition spec used  to write the manifest
    ///
    /// FIXME: we should parse this field.
    // pub partition_spec: Option<PartitionSpec>,

    /// ID of the partition spec used to write the manifest as a string
    pub partition_spec_id: i32,
    /// Table format version number of the manifest as a string
    pub format_version: i32,
    /// Type of content files tracked by the manifest: “data” or “deletes”
    pub content: ManifestContentType,
}

/// Type of content files tracked by the manifest
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ManifestContentType {
    Data,
    Deletes,
}

/// Used to track additions and deletions in ManifestEntry.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ManifestStatus {
    /// Value: 0
    Existing,
    /// Value: 1
    Added,
    /// Value: 2
    ///
    /// Deletes are informational only and not used in scans.
    Deleted,
}

/// Data file carries data file path, partition tuple, metrics, …
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataFile {
    /// field id: 134
    ///
    /// Type of content stored by the data file: data, equality deletes,
    /// or position deletes (all v1 files are data files)
    pub content: DataContentType,
    /// field id: 100
    ///
    /// Full URI for the file with FS scheme
    pub file_path: String,
    /// field id: 101
    ///
    /// String file format name, avro, orc or parquet
    pub file_format: DataFileFormat,
    /// field id: 102
    ///
    /// Partition data tuple, schema based on the partition spec output using
    /// partition field ids for the struct field ids
    ///
    /// TODO: we need to support partition in data file.
    pub partition: (),
    /// field id: 103
    ///
    /// Number of records in this file
    pub record_count: i64,
    /// field id: 104
    ///
    /// Total file size in bytes
    pub file_size_in_bytes: i64,
    /// field id: 108
    /// key field id: 117
    /// value field id: 118
    ///
    /// Map from column id to the total size on disk of all regions that
    /// store the column. Does not include bytes necessary to read other
    /// columns, like footers. Leave null for row-oriented formats (Avro)
    pub column_sizes: Option<HashMap<i32, i64>>,
    /// field id: 109
    /// key field id: 119
    /// value field id: 120
    ///
    /// Map from column id to number of values in the column (including null
    /// and NaN values)
    pub value_counts: Option<HashMap<i32, i64>>,
    /// field id: 110
    /// key field id: 121
    /// value field id: 122
    ///
    /// Map from column id to number of null values in the column
    pub null_value_counts: Option<HashMap<i32, i64>>,
    /// field id: 137
    /// key field id: 138
    /// value field id: 139
    ///
    /// Map from column id to number of NaN values in the column
    pub nan_value_counts: Option<HashMap<i32, i64>>,
    /// field id: 111
    /// key field id: 123
    /// value field id: 124
    ///
    /// Map from column id to number of distinct values in the column;
    /// distinct counts must be derived using values in the file by counting
    /// or using sketches, but not using methods like merging existing
    /// distinct counts
    pub distinct_counts: Option<HashMap<i32, i64>>,
    /// field id: 125
    /// key field id: 126
    /// value field id: 127
    ///
    /// Map from column id to lower bound in the column serialized as binary.
    /// Each value must be less than or equal to all non-null, non-NaN values
    /// in the column for the file.
    ///
    /// Reference:
    ///
    /// - [Binary single-value serialization](https://iceberg.apache.org/spec/#binary-single-value-serialization)
    pub lower_bounds: Option<HashMap<i32, Vec<u8>>>,
    /// field id: 128
    /// key field id: 129
    /// value field id: 130
    ///
    /// Map from column id to upper bound in the column serialized as binary.
    /// Each value must be greater than or equal to all non-null, non-Nan
    /// values in the column for the file.
    ///
    /// Reference:
    ///
    /// - [Binary single-value serialization](https://iceberg.apache.org/spec/#binary-single-value-serialization)
    pub upper_bounds: Option<HashMap<i32, Vec<u8>>>,
    /// field id: 131
    ///
    /// Implementation-specific key metadata for encryption
    pub key_metadata: Option<Vec<u8>>,
    /// field id: 132
    /// element field id: 133
    ///
    /// Split offsets for the data file. For example, all row group offsets
    /// in a Parquet file. Must be sorted ascending
    pub split_offsets: Vec<i64>,
    /// field id: 135
    /// element field id: 136
    ///
    /// Field ids used to determine row equality in equality delete files.
    /// Required when content is EqualityDeletes and should be null
    /// otherwise. Fields with ids listed in this column must be present
    /// in the delete file
    pub equality_ids: Option<Vec<i32>>,
    /// field id: 140
    ///
    /// ID representing sort order for this file.
    ///
    /// If sort order ID is missing or unknown, then the order is assumed to
    /// be unsorted. Only data files and equality delete files should be
    /// written with a non-null order id. Position deletes are required to be
    /// sorted by file and position, not a table order, and should set sort
    /// order id to null. Readers must ignore sort order id for position
    /// delete files.
    pub sort_order_id: Option<i32>,
}

/// Type of content stored by the data file: data, equality deletes, or
/// position deletes (all v1 files are data files)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataContentType {
    /// value: 0
    Data,
    /// value: 1
    PostionDeletes,
    /// value: 2
    EqualityDeletes,
}

/// Format of this data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataFileFormat {
    Avro,
    Orc,
    Parquet,
}