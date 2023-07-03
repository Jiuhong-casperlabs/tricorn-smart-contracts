// SPDX-License-Identifier: MIT
// vvv do we need to return comission to the user?
pragma solidity 0.8.19;

import { ECDSA } from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import { Errors } from  "../Errors.sol";

contract SignatureVerify {
    address private _signerAddress;

    constructor(address systemAddress_) {
        if (systemAddress_ == address(0)) {
            revert(Errors.INVALID_SIGNER_ADDRESS);
        }

        _signerAddress = systemAddress_;
    }

    function _checkBridgeInRequest(
        address senderAddress,
        address contractAddress,
        address token,
        uint256 amount,
        uint256 gasCommission,
        string memory destinationChain,
        string memory destinationAddress,
        uint256 deadline,
        uint256 nonce,
        uint256 transactionId,
        uint256 chainId,
        bytes calldata signature
    ) internal view {
        if (
            !_verify(
                _signerAddress,
                _hashBridgeIn(
                    senderAddress, 
                    contractAddress,
                    token,
                    amount, 
                    gasCommission, 
                    destinationChain, 
                    destinationAddress, 
                    deadline, 
                    nonce,
                    transactionId,
                    chainId
                ),
                signature
            )
        ) {
            revert("InvalidSignature");
        }
    }

    function _checkTransferOutRequest(
        address contractAddress,
        address token,
        address recipient,
        uint256 amount,
        uint256 commission,
        uint256 deadline,
        uint256 nonce,
        uint256 transactionId,
        uint256 chainId,
        bytes calldata signature
    ) internal view {
        if (
            !_verify(
                _signerAddress,
                _hashTransferOut(contractAddress, token, recipient, amount, commission, deadline, nonce, transactionId, chainId),
                signature
            )
        ) {
            revert("InvalidSignature");
        }
    }

    function _verify(
        address singerAddress,
        bytes32 hash,
        bytes calldata signature
    ) private pure returns (bool) {
        return singerAddress == ECDSA.recover(hash, signature);
    }

    function _hashBridgeIn(
        address senderAddress,
        address contractAddress,
        address token,
        uint256 amount,
        uint256 gasCommission,
        string memory destinationChain,
        string memory destinationAddress,
        uint256 deadline,
        uint256 nonce,
        uint256 transactionId,
        uint256 chainId
    ) private pure returns (bytes32) {
        return
            ECDSA.toEthSignedMessageHash(
                keccak256(
                    abi.encodePacked(
                        senderAddress, 
                        contractAddress,
                        token,
                        amount, 
                        gasCommission, 
                        destinationChain, 
                        destinationAddress, 
                        deadline, 
                        nonce,
                        transactionId,
                        chainId
                    )
                )
            );
    }

    function _hashTransferOut(
        address contractAddress,
        address token,
        address recipient,
        uint256 amount,
        uint256 commission,
        uint256 deadline,
        uint256 nonce,
        uint256 transactionId,
        uint256 chainId
    ) private pure returns (bytes32) {
        return
            ECDSA.toEthSignedMessageHash(
                keccak256(abi.encodePacked(contractAddress, token, recipient, amount, commission, deadline, nonce, transactionId, chainId))
            );
    }
}
