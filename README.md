# `zc-rlnc`

RLNC (Random Linear Network Coding) is an erasure coding scheme that allows for the reconstruction of original data from any threshold of coded packets as they arrive over the network.
Erasure coding can also be used to more efficiently transfer larger messages over multi-hop broadcast networks, because it allows for more efficient use of bandwidth. RLNC is especially suited for this use case because it assumes no specific network topology, and can be used in permissionless networks with high churn. 

## Overview
This project is part of **ZeroCast**, a Byzantine-resiliant P2P networking protocol. It therefore supports cryptographic security against [pollution attacks](https://en.wikipedia.org/wiki/Homomorphic_signatures_for_network_coding), assuming an honest source (broadcaster of the original message).

The implementation follows the design described in [this post](https://ethresear.ch/t/faster-block-blob-propagation-in-ethereum/21370).