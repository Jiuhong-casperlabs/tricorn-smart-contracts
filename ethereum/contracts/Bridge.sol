// SPDX-License-Identifier: UNLICENSED

pragma solidity 0.8.19;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {SafeERC20, IERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Pausable} from "@openzeppelin/contracts/security/Pausable.sol";
import {SignatureVerify} from "./signature/SignatureVerify.sol";
import {Errors} from "./Errors.sol";

contract Bridge is SignatureVerify, Ownable, Pausable {
    using SafeERC20 for IERC20;

    uint16 private constant _HUNDRED_PERCENT = 10_000;
    uint32 private constant _MAX_STABLE_COMMISSION_PERCENT = 90_00;

    uint32 private _stableCommissionPercent = 4_00;

    mapping(uint256 => bool) private _usedNonces;
    mapping(address => uint256) private _commissionPools;

    /// @param sender address who deposit tokens to the bridge
    /// @param nonce classic nonce parameter to track unique transaction
    /// @param token token we deposit to the bridge
    /// @param amount amount of this token
    /// @param stableCommissionPercent Commission percent which is actual on the moment when this event fired.
    /// @param gasCommission Gas commission on the destination chain which is actual when this event fired.
    /// @param destinationChain From what chain we transfer to the recipient
    /// @param destinationAddress From what address(in the above chain) we transfer to the recipient
    event BridgeFundsIn(
        address indexed sender,
        uint256 transactionId,
        uint256 nonce,
        address token,
        uint256 amount,
        uint256 stableCommissionPercent,
        uint256 gasCommission,
        string destinationChain,
        string destinationAddress
    );

    /// @param recipient recepient of the tokens
    /// @param token token we fund out from the bridge
    /// @param amount amount of this token
    /// @param transactionId heper parameter to track
    /// @param sourceChain From what chain we transfer to the recipient
    /// @param sourceAddress From what address(in the chain mentioned above) we transfer to the recipient
    event BridgeFundsOut(
        address indexed recipient,
        address token,
        uint256 amount,
        uint256 transactionId,
        string sourceChain,
        string sourceAddress
    );

    /// @param recipient recepient of the tokens (user who transfer his tokens out)
    /// @param nonce classic nonce parameter to track unique transaction
    /// @param token token we fund out from the bridge
    /// @param amount amount of this token
    event TransferOut(
        address indexed recipient,
        uint256 transactionId,
        uint256 nonce,
        address token,
        uint256 amount
    );

    /// @param token token we withdraw from the commission pool
    /// @param amount amount of this token
    event WithdrawCommission(address indexed token, uint256 amount);

    /// @notice bridgeIn method parameters
    /// @param token Token address
    /// @param amount Token amount
    /// @param gasCommission Commission which is calculated in transferred token.
    /// @param destinationChain Chain where we transfer tokens
    /// @param destinationAddress Address where we transfer tokens on the chain mentioned above
    /// @param deadline Timestamp until transaction is valid
    /// @param nonce Parameter to avoid repeat transaction attack
    struct BridgeInParams {
        address token;
        uint256 amount;
        uint256 gasCommission;
        string destinationChain;
        string destinationAddress;
        uint256 deadline;
        uint256 nonce;
        uint256 transactionId;
    }

    constructor(address signer) SignatureVerify(signer) {}

    /// @notice Deposit tokens on the bridge to transfer them onto another chain
    /// @dev Deposit tokens on the bridge to transfer them onto another chain
    function bridgeIn(
        BridgeInParams calldata params,
        bytes calldata signature
    ) external whenNotPaused {
        if (params.token == address(0)) {
            revert(Errors.INVALID_TOKEN_ADDRESS);
        }

        if (bytes(params.destinationAddress).length == 0) {
            revert(Errors.INVALID_DESTIONATION_ADDRESS);
        }

        if (bytes(params.destinationChain).length == 0) {
            revert(Errors.INVALID_DESTIONATION_CHAIN);
        }

        uint256 totalCommission = getTotalCommission(params.amount, params.gasCommission);

        if (totalCommission >= params.amount) {
            revert(Errors.COMMISSION_GREATER_THAN_AMOUNT);
        }

        if (_usedNonces[params.nonce]) {
            revert(Errors.ALREADY_USED_SIGNATURE);
        }

        if (block.timestamp > params.deadline) {
            revert(Errors.EXPIRED_SIGNATURE);
        }
        {
            _checkBridgeInRequest(
                _msgSender(),
                address(this),
                params.token,
                params.amount,
                params.gasCommission,
                params.destinationChain,
                params.destinationAddress,
                params.deadline,
                params.nonce,
                params.transactionId,
                getChainId(),
                signature
            );

            _usedNonces[params.nonce] = true;

            _commissionPools[params.token] += totalCommission;
        }

        // uint256 balanceBefore = IERC20(params.token).balanceOf(address(this));
        IERC20(params.token).safeTransferFrom(_msgSender(), address(this), params.amount);
        //uint256 transferredAmount = params.amount;

        emit BridgeFundsIn(
            _msgSender(),
            params.transactionId,
            params.nonce,
            params.token,
            params.amount,
            _stableCommissionPercent,
            params.gasCommission,
            params.destinationChain,
            params.destinationAddress
        );
    }

    /// @notice Withdraw tokens from the bridge. Can be initiated only by the owner
    /// @param token Token address
    /// @param recipient Recipient address
    /// @param amount Token amount
    /// @param transactionId ID of the transaction - helper parameter
    /// @param sourceChain From what chain we transfer to the recipient
    /// @param sourceAddress From what address(in the chain mentioned above) we transfer to the recipient
    function bridgeOut(
        address token,
        address recipient,
        uint256 amount,
        uint256 transactionId,
        string calldata sourceChain,
        string calldata sourceAddress
    ) external onlyOwner {
        if (recipient == address(0)) {
            revert(Errors.INVALID_RECIPIENT_ADDRESS);
        }

        if (token == address(0)) {
            revert(Errors.INVALID_TOKEN_ADDRESS);
        }

        uint256 balance = IERC20(token).balanceOf(address(this));
        uint256 allowedBalance = balance - _commissionPools[token];
        if (amount > allowedBalance) {
            revert(Errors.AMOUNT_EXCEED_BRIDGE_POOL);
        }
        IERC20(token).safeTransfer(recipient, amount);
        emit BridgeFundsOut(
            recipient,
            token,
            amount,
            transactionId,
            sourceChain,
            sourceAddress
        );
    }

    /// @notice Withdraw commission from the collected pool by the specified token.
    /// This way we do not affect user deposits as long as commission pool collected separately
    /// @param token Token address
    /// @param amount Token amount
    function withdrawCommission(
        address token,
        uint256 amount
    ) external onlyOwner {
        if (_commissionPools[token] < amount) {
            revert(Errors.AMOUNT_EXCEED_COMMISSION_POOL);
        }
        _commissionPools[token] -= amount;
        IERC20(token).safeTransfer(msg.sender, amount);
        emit WithdrawCommission(token, amount);
    }

    /// @notice Allow user to withdraw tokens back if the backend approved it by providing signature
    /// @param token Token address
    /// @param recipient Recipient address
    /// @param amount Token amount we should return back to the user
    /// @param commission Amount of commission we should return back to the user
    /// @param nonce Parameter to avoid double transaction attack
    /// @param signature Classic signature
    function transferOut(
        address token,
        address recipient,
        uint256 amount,
        uint256 commission,
        uint256 deadline,
        uint256 nonce,
        uint256 transactionId,
        bytes calldata signature
    ) external whenNotPaused {
        if (_usedNonces[nonce]) {
            revert(Errors.ALREADY_USED_SIGNATURE);
        }

        if (block.timestamp > deadline) {
            revert(Errors.EXPIRED_SIGNATURE);
        }

        if (token == address(0)) {
            revert(Errors.INVALID_TOKEN_ADDRESS);
        }

        if (recipient == address(0)) {
            revert(Errors.INVALID_RECIPIENT_ADDRESS);
        }

        _checkTransferOutRequest(
            address(this),
            token,
            recipient,
            amount,
            commission,
            deadline,
            nonce,
            transactionId,
            getChainId(),
            signature
        );

        _usedNonces[nonce] = true;
        _commissionPools[token] -= commission;

        uint256 totalSumForTransfer = amount + commission;
        IERC20(token).safeTransfer(recipient, totalSumForTransfer);

        // TODO:Descrease pool commission
        emit TransferOut(recipient, transactionId, nonce, token, totalSumForTransfer);
    }

    /// @notice Set stable commission percent which is used to calculate static commission Allowed only for onwer
    /// @param stableCommissionPercent_ percent
    function setStableCommissionPercent(
        uint32 stableCommissionPercent_
    ) external onlyOwner {
        if (stableCommissionPercent_ > _MAX_STABLE_COMMISSION_PERCENT) {
            revert(Errors.INVALID_STABLE_COMMISSION_PERCENT);
        }
        _stableCommissionPercent = stableCommissionPercent_;
    }

    /// @notice Stop all contract functionality allowed to the user
    function pause() external onlyOwner {
        _pause();
    }

    /// @notice Resume all contract functionality allowed to the user
    function unpause() external onlyOwner {
        _unpause();
    }

    /// @notice Get stable commission percent which is used to calculate static commission
    /// @return stable commission percent
    function getStableCommissionPercent() external view returns (uint256) {
        return _stableCommissionPercent;
    }

    /// @notice Get amount of collected commission by the specified token.
    /// @param token Specified token
    /// @return amount of collected commission
    function getCommissionPoolAmount(
        address token
    ) external view onlyOwner returns (uint256) {
        return _commissionPools[token];
    }

    /// @notice Block renounce ownership functionality
    function renounceOwnership() public view override onlyOwner {
        revert(Errors.INVALID_SIGNER_ADDRESS);
    }

    /// @notice Claculate total commission: stable commission percent + gasCommission
    /// @param amount Token amount
    /// @param gasCommission Manual gasCommission
    /// @return total commission user should pay for transfer
    function getTotalCommission(
        uint256 amount,
        uint256 gasCommission
    ) public view returns (uint256) {
        uint256 stableCommission = (amount * _stableCommissionPercent) /
            _HUNDRED_PERCENT;
        uint256 totalCommission = stableCommission + gasCommission;
        return totalCommission;
    }

    function getChainId() public view returns (uint256) {
        uint256 id;
        assembly {
            id := chainid()
        }
        return id;
    }
}
