use crate::data::config::BarNumber;
use ncm_api::ApiResponse;
use serde_json::Value;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    AuthorTileKind, HomeSidebarPlaylist, PlaylistTrack, PlaylistTrackKind,
    RecommendCard, SearchFilter, SearchItem, SEARCH_RESULT_PAGE_SIZE,
};

pub fn cycle_bar_number(current: BarNumber, delta: i32) -> BarNumber {
    let options = [
        BarNumber::Auto,
        BarNumber::N16,
        BarNumber::N32,
        BarNumber::N48,
        BarNumber::N64,
        BarNumber::N80,
        BarNumber::N96,
    ];
    let current_idx = options
        .iter()
        .position(|item| *item == current)
        .unwrap_or(0) as i32;
    let next = (current_idx + delta).rem_euclid(options.len() as i32) as usize;
    options[next]
}

pub fn parse_recommend_cards(response: &ApiResponse, limit: usize) -> Vec<RecommendCard> {
    response
        .body
        .get("recommend")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let title = item
                        .get("name")
                        .and_then(|value| value.as_str())?
                        .to_string();
                    let subtitle = first_non_empty(item, &["/copywriter", "/creator/nickname"])
                        .unwrap_or_else(|| "推荐歌单".to_string());
                    Some(RecommendCard {
                        id: parse_value_as_string(item.get("id")),
                        title,
                        subtitle,
                        cover_url: first_non_empty(item, &["/picUrl", "/coverImgUrl"]),
                    })
                })
                .take(limit.max(1))
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_personalized_cards(response: &ApiResponse, limit: usize) -> Vec<RecommendCard> {
    response
        .body
        .get("result")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let title = item
                        .get("name")
                        .and_then(|value| value.as_str())?
                        .to_string();
                    let subtitle = first_non_empty(item, &["/copywriter", "/creator/nickname"])
                        .unwrap_or_else(|| "推荐歌单".to_string());
                    Some(RecommendCard {
                        id: parse_value_as_string(item.get("id")),
                        title,
                        subtitle,
                        cover_url: first_non_empty(item, &["/picUrl", "/coverImgUrl"]),
                    })
                })
                .take(limit.max(1))
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_home_sidebar_playlists(response: &ApiResponse) -> Vec<HomeSidebarPlaylist> {
    let Some(items) = response
        .body
        .get("playlist")
        .and_then(|value| value.as_array())
        .or_else(|| {
            response
                .body
                .pointer("/data/list")
                .and_then(|value| value.as_array())
        })
        .or_else(|| {
            response
                .body
                .pointer("/data/playlist")
                .and_then(|value| value.as_array())
        })
    else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Some(title) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let track_count = item
            .get("trackCount")
            .and_then(|value| value.as_u64())
            .map(|value| value as usize)
            .or_else(|| {
                item.get("trackCount")
                    .and_then(|value| value.as_i64())
                    .map(|value| value.max(0) as usize)
            })
            .unwrap_or(0);

        let cover_url = parse_value_as_string(item.get("coverImgUrl"))
            .or_else(|| parse_value_as_string(item.get("picUrl")))
            .or_else(|| parse_value_as_string(item.get("coverUrl")));

        out.push(HomeSidebarPlaylist {
            id: parse_value_as_string(item.get("id")),
            title: title.to_string(),
            creator: item
                .pointer("/creator/nickname")
                .and_then(|value| value.as_str())
                .unwrap_or("Unknown User")
                .to_string(),
            track_count,
            cover_url,
        });
    }

    out
}

pub fn extract_current_user_id(response: &ApiResponse) -> Option<String> {
    for pointer in [
        "/profile/userId",
        "/data/profile/userId",
        "/account/id",
        "/data/account/id",
    ] {
        if let Some(value) = response.body.pointer(pointer) {
            if let Some(id) = parse_value_as_string(Some(value)) {
                if !id.trim().is_empty() {
                    return Some(id);
                }
            }
        }
    }

    None
}

pub fn extract_liked_playlist_id(response: &ApiResponse) -> Option<String> {
    for pointer in [
        "/profile/playlistId",
        "/data/profile/playlistId",
        "/profile/likesPlaylistId",
        "/data/profile/likesPlaylistId",
    ] {
        if let Some(value) = response.body.pointer(pointer) {
            if let Some(id) = parse_value_as_string(Some(value)) {
                if !id.trim().is_empty() {
                    return Some(id);
                }
            }
        }
    }

    None
}

pub fn extract_current_user_name(response: &ApiResponse) -> Option<String> {
    for pointer in [
        "/profile/nickname",
        "/data/profile/nickname",
        "/account/userName",
        "/data/account/userName",
    ] {
        if let Some(name) = response
            .body
            .pointer(pointer)
            .and_then(|value| value.as_str())
        {
            let name = name.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

pub fn parse_tracks(items: &[Value]) -> Vec<PlaylistTrack> {
    let mut tracks = Vec::new();

    for item in items {
        let Some(title) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let artist = parse_artists(item)
            .unwrap_or_else(|| "Unknown Artist".to_string())
            .trim()
            .to_string();
        let duration_ms = item
            .get("dt")
            .and_then(|value| value.as_i64())
            .or_else(|| item.get("duration").and_then(|value| value.as_i64()))
            .unwrap_or(0);

        tracks.push(PlaylistTrack {
            kind: PlaylistTrackKind::Song,
            id: parse_value_as_string(item.get("id")),
            title: title.to_string(),
            artist,
            album: item
                .pointer("/al/name")
                .and_then(|value| value.as_str())
                .unwrap_or("Unknown Album")
                .to_string(),
            cover_url: first_non_empty(item, &["/al/picUrl", "/album/picUrl"]),
            duration_ms,
            duration: format_duration(duration_ms),
        });
    }

    tracks
}

pub fn parse_song_like_check_result(body: &Value, song_id: &str) -> Option<bool> {
    if let Some(value) = body.pointer(&format!("/data/{song_id}")) {
        if let Some(liked) = parse_song_like_check_flag(value) {
            return Some(liked);
        }
    }

    for pointer in [
        "/data/0/liked",
        "/songs/0/liked",
        "/data/songs/0/liked",
        "/liked",
    ] {
        if let Some(value) = body.pointer(pointer) {
            if let Some(liked) = parse_song_like_check_flag(value) {
                return Some(liked);
            }
        }
    }

    if let Some(obj) = body.get("data").and_then(|value| value.as_object()) {
        for value in obj.values() {
            if let Some(liked) = parse_song_like_check_flag(value) {
                return Some(liked);
            }
        }
    }

    None
}

pub fn parse_song_like_check_flag(value: &Value) -> Option<bool> {
    if let Some(flag) = value.as_bool() {
        return Some(flag);
    }

    if let Some(flag) = value.as_i64() {
        return match flag {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
    }

    if let Some(flag) = value.as_u64() {
        return match flag {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
    }

    if let Some(flag) = value.as_str() {
        return match flag.trim().to_ascii_lowercase().as_str() {
            "0" | "false" => Some(false),
            "1" | "true" => Some(true),
            _ => None,
        };
    }

    None
}

pub fn parse_likelist_song_ids(body: &Value) -> HashSet<String> {
    let mut out = HashSet::new();
    let arrays = [
        body.pointer("/ids").and_then(|value| value.as_array()),
        body.pointer("/data/ids").and_then(|value| value.as_array()),
        body.pointer("/data").and_then(|value| value.as_array()),
    ];

    for maybe_arr in arrays {
        let Some(arr) = maybe_arr else {
            continue;
        };

        for item in arr {
            if let Some(value) = item
                .as_i64()
                .map(|value| value.to_string())
                .or_else(|| item.as_u64().map(|value| value.to_string()))
                .or_else(|| item.as_str().map(|value| value.trim().to_string()))
            {
                if !value.is_empty() {
                    out.insert(value);
                }
            }
        }
    }

    out
}

pub fn first_non_empty_intro_text(value: &Value) -> Option<String> {
    value
        .get("introduction")
        .and_then(|item| item.as_array())
        .and_then(|items| {
            items
                .iter()
                .find_map(|intro| first_non_empty(intro, &["/txt", "/ti"]))
        })
}

pub fn artist_album_kind(item: &Value) -> AuthorTileKind {
    let type_text = item
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let sub_type_text = item
        .get("subType")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    let type_lower = type_text.to_ascii_lowercase();
    let sub_type_lower = sub_type_text.to_ascii_lowercase();

    let is_single = type_lower.contains("single")
        || sub_type_lower.contains("single")
        || type_text.contains("单曲")
        || sub_type_text.contains("单曲");
    if is_single {
        return AuthorTileKind::Single;
    }

    let is_ep =
        type_lower.contains("ep") || sub_type_lower.contains("ep") || type_text.contains("EP");
    if is_ep {
        return AuthorTileKind::Ep;
    }

    AuthorTileKind::Album
}

pub fn parse_search_input(raw: &str) -> (String, SearchFilter) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), SearchFilter::Single);
    }

    let lower = trimmed.to_ascii_lowercase();

    for (suffix, filter) in [
        ("@single", SearchFilter::Single),
        ("@album", SearchFilter::Album),
        ("@author", SearchFilter::Author),
        ("@artist", SearchFilter::Author),
        ("@list", SearchFilter::Playlist),
    ] {
        if lower.ends_with(suffix) {
            let cut = trimmed.len().saturating_sub(suffix.len());
            let stripped = &trimmed[..cut];
            return (stripped.trim().to_string(), filter);
        }
    }

    (trimmed.to_string(), SearchFilter::Single)
}

pub fn is_followed_author_query(keywords: &str, filter: SearchFilter) -> bool {
    filter == SearchFilter::Author && keywords.trim().is_empty()
}

pub struct FollowedAuthorPage {
    pub items: Vec<SearchItem>,
    pub fetched_count: usize,
    pub has_more: Option<bool>,
    pub total_count: Option<usize>,
}

pub fn parse_followed_author_page(response: &ApiResponse) -> FollowedAuthorPage {
    let items = response
        .body
        .get("data")
        .and_then(|value| value.as_array())
        .or_else(|| {
            response
                .body
                .get("artists")
                .and_then(|value| value.as_array())
        })
        .or_else(|| {
            response
                .body
                .pointer("/result/artists")
                .and_then(|value| value.as_array())
        });

    let fetched_count = items.map(|values| values.len()).unwrap_or_default();
    let parsed_items = items
        .map(|values| parse_author_items(values))
        .unwrap_or_default();

    let has_more = ["/hasMore", "/more", "/data/hasMore", "/result/hasMore"]
        .iter()
        .find_map(|pointer| {
            response
                .body
                .pointer(pointer)
                .and_then(|value| value.as_bool())
        });
    let total_count = ["/count", "/data/count", "/result/count"]
        .iter()
        .find_map(|pointer| parse_usize_value(response.body.pointer(pointer)));

    FollowedAuthorPage {
        items: parsed_items,
        fetched_count,
        has_more,
        total_count,
    }
}

pub fn followed_author_has_more(page: &FollowedAuthorPage, next_offset: usize) -> bool {
    if page.fetched_count == 0 {
        return false;
    }

    if let Some(has_more) = page.has_more {
        return has_more;
    }

    if let Some(total_count) = page.total_count {
        return next_offset < total_count;
    }

    page.fetched_count >= SEARCH_RESULT_PAGE_SIZE
}

pub fn parse_usize_value(value: Option<&Value>) -> Option<usize> {
    let value = value?;
    if let Some(number) = value.as_u64() {
        return Some(number as usize);
    }
    if let Some(number) = value.as_i64() {
        if number >= 0 {
            return Some(number as usize);
        }
    }
    if let Some(text) = value.as_str() {
        return text.trim().parse::<usize>().ok();
    }
    None
}

pub fn parse_search_items(response: &ApiResponse, filter: SearchFilter) -> Vec<SearchItem> {
    let Some(result) = response.body.get("result") else {
        return Vec::new();
    };

    match filter {
        SearchFilter::Single => result
            .get("songs")
            .and_then(|value| value.as_array())
            .map(|items| parse_song_items(items))
            .unwrap_or_default(),
        SearchFilter::Album => result
            .get("albums")
            .and_then(|value| value.as_array())
            .map(|items| parse_album_items(items))
            .unwrap_or_default(),
        SearchFilter::Author => result
            .get("artists")
            .and_then(|value| value.as_array())
            .map(|items| parse_author_items(items))
            .unwrap_or_default(),
        SearchFilter::Playlist => result
            .get("playlists")
            .and_then(|value| value.as_array())
            .map(|items| parse_playlist_items(items))
            .unwrap_or_default(),
    }
}

pub fn parse_song_items(items: &[Value]) -> Vec<SearchItem> {
    let mut out = Vec::new();

    for item in items {
        let Some(name) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let artist = parse_artists(item).unwrap_or_else(|| "Unknown Artist".to_string());
        let duration = item
            .get("dt")
            .and_then(|value| value.as_i64())
            .or_else(|| item.get("duration").and_then(|value| value.as_i64()))
            .unwrap_or(0);

        out.push(SearchItem {
            left_label: format!("{} - {}", name, artist),
            right_label: format_duration(duration),
            type_tag: None,
            song_id: parse_value_as_string(item.get("id")),
            album_id: None,
            playlist_id: None,
            artist_id: None,
            title: Some(name.to_string()),
            artist: Some(artist),
            album: item
                .pointer("/al/name")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string()),
            cover_url: first_non_empty(item, &["/al/picUrl", "/album/picUrl"]),
            duration_ms: Some(duration),
        });
    }

    out
}

pub fn parse_album_items(items: &[Value]) -> Vec<SearchItem> {
    let mut out = Vec::new();

    for item in items {
        let Some(name) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let artist = item
            .pointer("/artist/name")
            .and_then(|value| value.as_str())
            .unwrap_or("Unknown Artist");
        let size = item
            .get("size")
            .and_then(|value| value.as_i64())
            .unwrap_or_default();

        out.push(SearchItem {
            left_label: format!("{} - {}", name, artist),
            right_label: format!("{} 首", size),
            type_tag: Some("@album".to_string()),
            song_id: None,
            album_id: parse_value_as_string(item.get("id")),
            playlist_id: None,
            artist_id: None,
            title: Some(name.to_string()),
            artist: Some(artist.to_string()),
            album: Some(name.to_string()),
            cover_url: first_non_empty(item, &["/picUrl", "/blurPicUrl"]),
            duration_ms: None,
        });
    }

    out
}

pub fn parse_author_items(items: &[Value]) -> Vec<SearchItem> {
    let mut out = Vec::new();

    for item in items {
        let Some(name) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let album_size = item
            .get("albumSize")
            .and_then(|value| value.as_i64())
            .unwrap_or_default();

        out.push(SearchItem {
            left_label: name.to_string(),
            right_label: format!("{} 张专辑", album_size),
            type_tag: Some("@author".to_string()),
            song_id: None,
            album_id: None,
            playlist_id: None,
            artist_id: parse_value_as_string(item.get("id")),
            title: None,
            artist: Some(name.to_string()),
            album: None,
            cover_url: first_non_empty(item, &["/picUrl", "/img1v1Url", "/avatarUrl"]),
            duration_ms: None,
        });
    }

    out
}

pub fn parse_playlist_items(items: &[Value]) -> Vec<SearchItem> {
    let mut out = Vec::new();

    for item in items {
        let Some(name) = item.get("name").and_then(|value| value.as_str()) else {
            continue;
        };

        let creator = item
            .pointer("/creator/nickname")
            .and_then(|value| value.as_str())
            .unwrap_or("Unknown User");
        let count = item
            .get("trackCount")
            .and_then(|value| value.as_i64())
            .unwrap_or_default();

        out.push(SearchItem {
            left_label: format!("{} - {}", name, creator),
            right_label: format!("{} 首", count),
            type_tag: Some("@list".to_string()),
            song_id: None,
            album_id: None,
            playlist_id: parse_value_as_string(item.get("id")),
            artist_id: None,
            title: None,
            artist: None,
            album: None,
            cover_url: first_non_empty(item, &["/coverImgUrl", "/picUrl"]),
            duration_ms: None,
        });
    }

    out
}

pub fn parse_artists(track: &Value) -> Option<String> {
    let artists = track
        .get("ar")
        .and_then(|value| value.as_array())
        .or_else(|| track.get("artists").and_then(|value| value.as_array()))?;

    let names: Vec<String> = artists
        .iter()
        .filter_map(|item| item.get("name").and_then(|value| value.as_str()))
        .map(|name| name.to_string())
        .collect();

    if names.is_empty() {
        None
    } else {
        Some(names.join(" / "))
    }
}

pub fn format_duration(duration_ms: i64) -> String {
    let total = (duration_ms.max(0) / 1000) as u64;
    let mm = total / 60;
    let ss = total % 60;
    format!("{:02}:{:02}", mm, ss)
}

pub fn parse_value_as_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    if let Some(number) = value.as_i64() {
        return Some(number.to_string());
    }
    None
}

pub fn first_non_empty(value: &Value, pointers: &[&str]) -> Option<String> {
    for pointer in pointers {
        if let Some(text) = value.pointer(pointer).and_then(|item| item.as_str()) {
            let text = text.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }

    None
}

pub fn response_indicates_vip(response: &ApiResponse) -> bool {
    let code = response
        .body
        .get("code")
        .and_then(|value| value.as_i64())
        .unwrap_or(response.status);
    if code != 200 {
        return false;
    }

    let root = response.body.get("data").unwrap_or(&response.body);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    for pointer in [
        "/redVipLevel",
        "/redplusLevel",
        "/musicPackage/vipCode",
        "/associator/vipCode",
        "/musicVipLevel",
    ] {
        if root
            .pointer(pointer)
            .and_then(|value| value.as_i64())
            .unwrap_or(0)
            > 0
        {
            return true;
        }
    }

    for pointer in [
        "/vipStatus",
        "/musicPackage/isSign",
        "/associator/isSign",
        "/isVip",
    ] {
        if root
            .pointer(pointer)
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return true;
        }
    }

    let music_expire = root
        .pointer("/musicPackage/expireTime")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    if music_expire > now_ms {
        return true;
    }

    let associator_expire = root
        .pointer("/associator/expireTime")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    if associator_expire > now_ms {
        return true;
    }

    false
}
