namespace rs volo.example

struct Item {
    1: required i64 id,
    2: required string title,
    3: required string content,

    10: optional map<string, string> extra,
}

struct KV {
    1: required string key,
    2: required string value,
}

struct PostItemRequest {
    1: required string name,
}

struct PostItemResponse {
    1: required Item item,
}

struct GetItemRequest {
    1: required string key,
}

struct GetItemResponse {
    1: required string value,
}

struct SetItemRequest {
    1: required KV kv,
}

struct SetItemResponse {
    1: required string message,
}

struct DeleteItemRequest {
    1: required list<string> keys,
}

struct DeleteItemResponse {
    1: required i64 count,
}

struct PingRequest {
    1: optional string message,
}

struct PingResponse {
    1: required string message,
}





service ItemService {
    PostItemResponse PostItem (1: PostItemRequest req),
    GetItemResponse GetItem (1: GetItemRequest req),
    SetItemResponse SetItem (1: SetItemRequest req),
    DeleteItemResponse DeleteItem (1: DeleteItemRequest req),
    PingResponse Ping (1: PingRequest req),
}
