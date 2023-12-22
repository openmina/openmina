## Sync Staged Ledger

At this point, we already have synced snarked ledger that the staged
ledger builds on top of. Now we need to:
1. Fetch additional parts (scan state, pending coinbases, etc...)
   necessary for reconstructing staged ledger.
2. Use fetched parts along with already synced snarked ledger in order to
   reconstruct staged ledger.

```mermaid
flowchart
subgraph Services and event source
P2pService([P2pService])
LedgerService([LedgerService])
event_source[(event_source)]
end

subgraph 1. Fetch valid staged ledger parts
StagedPartsPeerFetchInit>StagedPartsPeerFetchInit]
P2P-- peer available -->StagedPartsPeerFetchInit
StagedPartsPeerFetchInit-- "call" rpc -->P2P
StagedPartsPeerFetchInit-->StagedPartsPeerFetchPending
P2P-. rpc timed out, peer disconnected, etc... -.->StagedPartsPeerFetchError
StagedPartsPeerFetchError-. retry from another peer -.->StagedPartsPeerFetchInit
P2P-- rpc response -->StagedPartsPeerFetchSuccess
StagedPartsPeerFetchSuccess-.->StagedPartsPeerInvalid
StagedPartsPeerInvalid-. retry from another peer .->StagedPartsPeerFetchInit
StagedPartsPeerFetchSuccess-->StagedPartsPeerValid
StagedPartsPeerValid-->StagedPartsFetchSuccess
P2P-- initiate rpc request -->P2pService
P2pService-- send received rpc response --> event_source
event_source-- rpc response received -->P2P
end

subgraph 2. Reconstruct staged ledger
wait(can't proceed! wait for new best tip.)
cont(continue bootstrap/catchup)
StagedReconstructInit>StagedReconstructInit]
StagedPartsFetchSuccess-->StagedReconstructInit
StagedReconstructInit-- initiate reconstruction -->LedgerService
StagedReconstructInit-->StagedReconstructPending
LedgerService-- send result --> event_source
event_source-.->StagedReconstructError--->wait
event_source--->StagedReconstructSuccess-->cont
end
