// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../HoprChannels.sol";

contract TicketsMock is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}

    function fundChannelInternal(
        address funder,
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        _fundChannel(accountA, accountB, amountA, amountB);
    }

    function initiateChannelClosureInternal(
        address initiator,
        address counterparty
    ) external {
        _initiateChannelClosure(initiator, counterparty);
    }

    function finalizeChannelClosureInternal(
        address initiator,
        address counterparty
    ) external {
        _finalizeChannelClosure(initiator, counterparty);
    }

    function redeemTicketInternal(
        address recipient,
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        _redeemTicket(
            msg.sender,
            counterparty,
            nextCommitment,
            ticketEpoch,
            ticketIndex,
            proofOfRelaySecret,
            amount,
            winProb,
            r,
            s,
            v
        );
    }

    function getEncodedTicketInternal(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) external pure returns (bytes memory) {
        return _getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, winProb);
    }

    function getTicketHashInternal(
        bytes calldata packedTicket
    ) external pure returns (bytes32) {
        return _getTicketHash(packedTicket);
    }

    function getTicketLuckInternal(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) external pure returns (bytes32) {
        return _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret, winProb);
    }
}
