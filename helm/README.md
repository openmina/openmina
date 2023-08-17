# Helm Charts for Openmina

## Runing Openmina Snarker for Berkeley Testnet

This will start openmina snarker using account, chain ID and fee as specified in [openmina/values.yaml].
``` sh
helm install openmina ./openmina
```

To use e.g. different account and fee, run

``` sh
helm install openmina ./openmina --set=openmina.snarkerPublicKey=<pubkey> --set=openmina.fee=<fee>
```

## Runnig Openmina Nodes with Bootstrap Replayer

The command [bootstrap.sh] prints Helm commands that should be executed to install all needed pieces:

``` sh
$  ./bootstrap.sh 
helm upgrade --install bootstrap-replayer ./bootstrap-replayer
helm upgrade --install openmina1 ./openmina --set=openmina.peers="/dns4/bootstrap-bootstrap-replayer/tcp/8302/p2p/12D3KooWETkiRaHCdztkbmrWQTET9HMWimQPx5sH5pLSRZNxRsjw /2axsdDAiiZee7hUsRPMtuyHt94UMrvJmMQDhDjKhdRhgqkMdy8e/http/openmina1/3000 /2bpACUcRh2u7WJ3zSBRWZZvQMTMofYr9SGQgcP2YKzwwDKanNAy/http/openmina2/3000 /2aQA3swTKVf16YgLXZS7TizU7ASgZ8LidEgyHhChpDinrvM9NMi/http/openmina3/3000" --set=openmina.secretKey=5KJKg7yAbYAQcNGWcKFf2C4ruJxwrHoQvsksU16yPzFzXHMsbMc
helm upgrade --install openmina2 ./openmina --set=openmina.peers="/dns4/bootstrap-bootstrap-replayer/tcp/8302/p2p/12D3KooWETkiRaHCdztkbmrWQTET9HMWimQPx5sH5pLSRZNxRsjw /2axsdDAiiZee7hUsRPMtuyHt94UMrvJmMQDhDjKhdRhgqkMdy8e/http/openmina1/3000 /2bpACUcRh2u7WJ3zSBRWZZvQMTMofYr9SGQgcP2YKzwwDKanNAy/http/openmina2/3000 /2aQA3swTKVf16YgLXZS7TizU7ASgZ8LidEgyHhChpDinrvM9NMi/http/openmina3/3000" --set=openmina.secretKey=5JgkZGzHPC2SmQqRGxwbFjZzFMLvab5tPwkiN29HX9Vjc9rtwV4
helm upgrade --install openmina3 ./openmina --set=openmina.peers="/dns4/bootstrap-bootstrap-replayer/tcp/8302/p2p/12D3KooWETkiRaHCdztkbmrWQTET9HMWimQPx5sH5pLSRZNxRsjw /2axsdDAiiZee7hUsRPMtuyHt94UMrvJmMQDhDjKhdRhgqkMdy8e/http/openmina1/3000 /2bpACUcRh2u7WJ3zSBRWZZvQMTMofYr9SGQgcP2YKzwwDKanNAy/http/openmina2/3000 /2aQA3swTKVf16YgLXZS7TizU7ASgZ8LidEgyHhChpDinrvM9NMi/http/openmina3/3000" --set=openmina.secretKey=5KWkmiairnLJjtvqEatpb4grLEG8oZjFp7ye4ehphjXRGrgsuH8
```
