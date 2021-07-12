use crate::validate::Response;

pub fn add_visualization(
    response: &mut Response,
    raw_gtfs: Result<gtfs_structures::RawGtfs, gtfs_structures::Error>,
) -> &Response {
    response
}
