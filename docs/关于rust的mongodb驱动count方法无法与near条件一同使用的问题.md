最近在自己的项目中使用 mongodb，遇到了一个比较诡异的问题， 就是如果通过 rust mongodb driver 来对 mongodb 进行操作, 在用 db.collection.count()方法的时候， 如果查询条件里面包含$near， 则会报`$geoNear, $near, and $nearSphere are not allowed in this context`, 一句话没头没尾， 我也搞不清这里面提到的上下文具体指什么， 网上搜了一下， 相关的信息比较少， 大部分也没有正面的回答是哪里的问题，有解决方案的更少。没办法，只能自己试了，通过 mongo shell 执行该查询是可以成功拿到返回值的， 所以基本可以确定是 rust 的 mongodb driver 的问题， 目前版本是 2.3.0， 已经是最新的了， 解决的方法只能是曲线救国了， 将 db.collection.count()改成 db.collection.find(), 因为 db.collection.find()返回的是一个 Stream, futures 这个 crate 提供了对 Stream 的扩展(StreamExt), 里面包括一个 count()方法， 我们可以直接调用得到计数。

但是注意， **_因为 Stream 本质上是一个 Iterator, 所以我们这个计数真的是遍历整个结果集得出的， 不管是从时间来讲还是从资源开销来将， 成本都非常高_**， 因为我的应用场景一般计数不会超过 7 个， 所以我勉强可以接受， 如果计数较大的场景， 请另寻出路

---

**2022-08-11 更新**
现在发现如果使用 database.run_command 是可以使用$near 条件的， 代码如下:

```rust
        let res = db
            .run_command(
                doc! {
                    "count": "locations",
                    "query": doc!{
                            "$and": vec![
                                doc!{ "geo_index": doc!{ "$in": vec![613362111795429375i64] }},
                                doc!{ "location": {
                                    "$near": doc!{
                                    "$geometry": {
                                        "type": "Point",
                                        "coordinates": vec![117, 36],
                                    },
                                    "$maxDistance": 100000
                                }}}
                            ]
                    }
                },
                None,
            )
            .await
            .unwrap();
        let count = res.get_i32("n").unwrap();
```
