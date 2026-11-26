//! The reputation: the client's tables of the reputations and of their groups.

use super::*;

/// One document of a BSON file as JSON (the client reads its `ContentData` files with a JSON reader after
/// unpacking them: `JsonCppLoader::Load`): texts, whole numbers, numbers with a fraction, yes and no, nothing,
/// documents and lists; anything else ends the reading, as does a file that is cut short.
fn document(bytes: &[u8], list: bool) -> Option<serde_json::Value> {
    let length = usize::try_from(i32::from_le_bytes(bytes.get(..4)?.try_into().ok()?)).ok()?;
    let mut rest = bytes.get(4..length.checked_sub(1)?)?;
    let (mut fields, mut items) = (serde_json::Map::new(), Vec::new());
    while let Some((&kind, after)) = rest.split_first() {
        let end = after.iter().position(|byte| *byte == 0)?;
        let name = String::from_utf8_lossy(&after[..end]).into_owned();
        let value = after.get(end + 1..)?;
        let sized = |size: usize| value.get(..size);
        let inner = || usize::try_from(i32::from_le_bytes(value.get(..4)?.try_into().ok()?)).ok();
        let (found, took): (serde_json::Value, usize) = match kind {
            0x01 => (f64::from_le_bytes(sized(8)?.try_into().ok()?).into(), 8),
            0x02 => {
                let size = inner()?;
                (String::from_utf8_lossy(value.get(4..(4 + size).checked_sub(1)?)?).into_owned().into(), 4 + size)
            }
            0x03 | 0x04 => (document(value, kind == 0x04)?, inner()?),
            0x08 => ((*value.first()? != 0).into(), 1),
            0x0a => (serde_json::Value::Null, 0),
            0x10 => (i32::from_le_bytes(sized(4)?.try_into().ok()?).into(), 4),
            0x12 => (i64::from_le_bytes(sized(8)?.try_into().ok()?).into(), 8),
            _ => return None,
        };
        if list {
            items.push(found);
        } else {
            fields.insert(name, found);
        }
        rest = value.get(took..)?;
    }
    Some(if list { items.into() } else { fields.into() })
}

/// The entries of one of the two files' tables, by their numbers and in their order (the client keeps each
/// in a map by the number its key is: `JsonCppLoader::GetReputeInfoData`).
fn entries<'file>(file: &'file serde_json::Value, table: &str) -> Vec<(i64, &'file serde_json::Value)> {
    by_number(&file[table])
}

/// The two tables as JSON, out of the files' documents.
fn told(infos: &serde_json::Value, groups: &serde_json::Value) -> serde_json::Value {
    let whole = |value: &serde_json::Value| value.as_i64().or_else(|| value.as_f64().map(|number| number as i64)).unwrap_or(0).max(0);
    let reputes: Vec<serde_json::Value> = entries(infos, "reputeInfo")
        .into_iter()
        .map(|(id, info)| {
            serde_json::json!({
                "id": id, "name": info["Name"].as_str().unwrap_or_default(), "most": whole(&info["MaxPoint_Positive"]),
                "least": whole(&info["MaxPoint_Negative"]), "seen": info["Invisible"].as_str().unwrap_or_default(),
            })
        })
        .collect();
    let groups: Vec<serde_json::Value> = entries(groups, "ReputeGroup")
        .into_iter()
        .map(|(id, group)| {
            let reputes: Vec<i64> = group["ReputeList"].as_array().map(|list| list.iter().map(whole).collect()).unwrap_or_default();
            serde_json::json!({ "id": id, "name": group["Name"].as_str().unwrap_or_default(), "reputes": reputes })
        })
        .collect();
    serde_json::json!({ "reputes": reputes, "groups": groups })
}

/// `{"reputes": [{"id": 1, "name": "Orc Village", "most": 3000, "least": 3000, "seen": "VISIBLE_TRUE"}, …],
/// "groups": [{"id": 1, "name": "Friends", "reputes": [1, 2]}, …]}`: the client's reputations and the groups
/// its window sorts them by, each in the order of its numbers (`data\ContentData\Repute\ReputeInfoData.bson`
/// and `ReputeGroupData.bson`, read by `CReputeMgr::Load`; their texts are UTF-8). `seen` is the table's
/// `Invisible` as it is written.
pub fn reputation(grf: &Grf) -> serde_json::Value {
    let file = |name: &str| {
        let path = format!("data\\contentdata\\repute\\{name}.bson");
        let read = grf.read(&path).map_err(|error| error.to_string()).and_then(|bytes| document(&bytes, false).ok_or_else(|| "no BSON".to_owned()));
        read.unwrap_or_else(|error| {
            eprintln!("tables: {path}: {error}");
            serde_json::Value::Null
        })
    };
    told(&file("reputeinfodata"), &file("reputegroupdata"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(name: &str, value: &str) -> Vec<u8> {
        [&[2u8][..], name.as_bytes(), &[0], &(value.len() as i32 + 1).to_le_bytes(), value.as_bytes(), &[0]].concat()
    }

    fn number(name: &str, value: i32) -> Vec<u8> {
        [&[0x10u8][..], name.as_bytes(), &[0], &value.to_le_bytes()].concat()
    }

    fn nested(kind: u8, name: &str, fields: &[Vec<u8>]) -> Vec<u8> {
        [&[kind][..], name.as_bytes(), &[0], &whole(fields)].concat()
    }

    fn whole(fields: &[Vec<u8>]) -> Vec<u8> {
        let body = fields.concat();
        [&(body.len() as i32 + 5).to_le_bytes()[..], &body, &[0]].concat()
    }

    #[test]
    fn a_bson_document_is_read_as_json() {
        let file = whole(&[nested(3, "ReputeGroup", &[nested(3, "2", &[text("Name", "Isgard"), nested(4, "ReputeList", &[number("0", 4), number("1", 7)])])])]);
        assert_eq!(document(&file, false), Some(serde_json::json!({ "ReputeGroup": { "2": { "Name": "Isgard", "ReputeList": [4, 7] } } })));
        assert_eq!(document(&file[..file.len() - 3], false), None);
        assert_eq!(document(&whole(&[vec![0x7f, b'a', 0]]), false), None);
    }

    #[test]
    fn the_tables_are_listed_by_their_numbers() {
        let infos = serde_json::json!({ "reputeInfo": {
            "10": { "Name": "Golden Root", "MaxPoint_Positive": 5000, "MaxPoint_Negative": 0, "Invisible": "VISIBLE_EXIST" },
            "2": { "Name": "Goblin Village", "MaxPoint_Positive": 3000, "MaxPoint_Negative": 3000, "Invisible": "VISIBLE_TRUE" },
        } });
        let groups = serde_json::json!({ "ReputeGroup": { "1": { "ID": "MonsterGroup1", "Name": "Friends", "ReputeList": [1, 2] } } });
        let told = told(&infos, &groups);
        assert_eq!(told["reputes"][0], serde_json::json!({ "id": 2, "name": "Goblin Village", "most": 3000, "least": 3000, "seen": "VISIBLE_TRUE" }));
        assert_eq!(told["reputes"][1]["id"], 10);
        assert_eq!(told["groups"], serde_json::json!([{ "id": 1, "name": "Friends", "reputes": [1, 2] }]));
        assert_eq!(super::told(&serde_json::Value::Null, &serde_json::Value::Null), serde_json::json!({ "reputes": [], "groups": [] }));
    }
}
