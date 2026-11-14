use super::prelude::*;
use super::session::{encode_frame, remote_point};

#[test]
fn encode_frame_rejects_truncated_input() {
    assert!(encode_frame(2, 2, ImageFormat::ARGB, 1, &[0; 4]).is_none());
}

#[test]
fn encode_frame_writes_a_png_for_argb() {
    let png = encode_frame(1, 1, ImageFormat::ARGB, 1, &[0, 0, 255, 255])
        .expect("a complete pixel should encode");
    assert!(png.starts_with("data:image/png;base64,"));
    assert!(png.len() > "data:image/png;base64,".len());
}

#[test]
fn remote_point_scales_to_the_remote_frame() {
    let state = DioxusUiState {
        width: 1_920,
        height: 1_080,
        viewport_width: 960.0,
        viewport_height: 540.0,
        ..Default::default()
    };
    assert_eq!(remote_point(480.0, 270.0, &state), (960, 540));
    assert_eq!(remote_point(2_000.0, 600.0, &state), (1_919, 1_079));
}

#[test]
fn numeric_remote_ids_are_grouped_and_normalized_for_connection() {
    assert_eq!(format_remote_id("123456789"), "123 456 789");
    assert_eq!(format_remote_id("123456/r"), "123 456/r");
    assert_eq!(normalize_remote_id("123 456 789"), "123456789");
}

#[test]
fn non_numeric_remote_ids_keep_their_original_format() {
    assert_eq!(format_remote_id("desk- name /r"), "desk- name /r");
}

#[test]
fn peer_sort_matches_flutter_preferences_and_preserves_recent_order() {
    let mut peers = vec![
        PeerCardData {
            id: "z".to_owned(),
            alias: "Zulu".to_owned(),
            hostname: "workstation".to_owned(),
            username: "zoe".to_owned(),
            ..Default::default()
        },
        PeerCardData {
            id: "a".to_owned(),
            alias: "Alpha".to_owned(),
            hostname: "laptop".to_owned(),
            username: "alice".to_owned(),
            online: true,
            ..Default::default()
        },
    ];

    sort_peer_cards(&mut peers, "recent", PEER_SORT_REMOTE_ID);
    assert_eq!(peers[0].id, "z");

    sort_peer_cards(&mut peers, "favorites", PEER_SORT_REMOTE_HOST);
    assert_eq!(peers[0].id, "a");
    sort_peer_cards(&mut peers, "favorites", PEER_SORT_USERNAME);
    assert_eq!(peers[0].id, "a");
    sort_peer_cards(&mut peers, "favorites", PEER_SORT_STATUS);
    assert_eq!(peers[0].id, "a");
    assert_eq!(normalize_peer_sort("online"), PEER_SORT_STATUS);
    assert_eq!(normalize_peer_sort("name"), PEER_SORT_REMOTE_ID);
}
