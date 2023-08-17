#!/usr/bin/env sh

helm=$(dirname $0)

REPLAYER=/dns4/bootstrap-bootstrap-replayer/tcp/8302/p2p/12D3KooWETkiRaHCdztkbmrWQTET9HMWimQPx5sH5pLSRZNxRsjw
NAME=openmina
PORT=3000

NODES="5KJKg7yAbYAQcNGWcKFf2C4ruJxwrHoQvsksU16yPzFzXHMsbMc:2axsdDAiiZee7hUsRPMtuyHt94UMrvJmMQDhDjKhdRhgqkMdy8e:B62qqYvLLtTMQtHxRfuzZK21AJrqFE8Zq9Cyk3wtjegiTRn5soNQA9A\
    5JgkZGzHPC2SmQqRGxwbFjZzFMLvab5tPwkiN29HX9Vjc9rtwV4:2bpACUcRh2u7WJ3zSBRWZZvQMTMofYr9SGQgcP2YKzwwDKanNAy:B62qrV28zSmLjxMZP1jKRSEFsajPGdFRukbvnXzRKyDmUBNVvCH7w9o\
    5KWkmiairnLJjtvqEatpb4grLEG8oZjFp7ye4ehphjXRGrgsuH8:2aQA3swTKVf16YgLXZS7TizU7ASgZ8LidEgyHhChpDinrvM9NMi:B62qkgVSEnzTabaFJzZcG1pgXorRLJREJFvchGya6UGoKTmFx5AWAK6"

peers() {
    PEERS=$REPLAYER
    N=1
    for NODE in $*; do
        PK=$(echo $NODE | sed -e "s/.*:\(.*\):.*/\1/")
        PEERS="$PEERS /$PK/http/$NAME$N/$PORT"
        N=$((N+1))
    done
    echo $PEERS
}

echo helm upgrade --install bootstrap-replayer $helm/bootstrap-replayer
N=1
for NODE in $NODES; do
    SK=${NODE%%:*}
    NODE=${NODE#*:}
    PK=${NODE%%:*}
    WK=${NODE#*:}

    PEERS=$(peers $NODES)
    echo helm upgrade --install openmina$N $helm/openmina --set=openmina.peers=\"$PEERS\" --set=openmina.secretKey="$SK"

    N=$((N+1))
done
