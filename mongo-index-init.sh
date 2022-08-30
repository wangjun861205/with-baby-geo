#!/bin/sh

mongo <<EOF
use with-baby-geo;
db.locations.createIndex({location: "2dsphere"});
EOF

