table! {
    host_log (hostname, timestamp) {
        hostname -> Varchar,
        status -> Nullable<Integer>,
        timestamp -> Datetime,
        ipv4 -> Nullable<Varchar>,
        ipv6 -> Nullable<Varchar>,
        mac -> Nullable<Varchar>,
    }
}

table! {
    sensor_log (sensor_id, timestamp) {
        sensor_id -> Integer,
        sensor_name -> Nullable<Varchar>,
        timestamp -> Datetime,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
    }
}

table! {
    sensor_log2 (sensor_id, timestamp) {
        sensor_id -> Integer,
        sensor_name -> Nullable<Varchar>,
        timestamp -> Datetime,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
    }
}

allow_tables_to_appear_in_same_query!(
    host_log,
    sensor_log,
    sensor_log2,
);
