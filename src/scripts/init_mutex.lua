local now = redis.call("time")[1];
local prev = redis.call("GETSET", KEYS[1], now);
if prev == nil or now - prev >= ARGV[1] {
    return
}
redis.call("SET", KEYS[1], prev);
