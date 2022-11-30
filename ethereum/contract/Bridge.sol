// SPDX-License-Identifier: UNLICENSED

pragma solidity ^0.8.0;

import "./IERC20.sol";
import "./SafeERC20.sol";

contract Bridge {
    using SafeERC20 for IERC20;

    event BridgeFundsIn(
        address token,
        address sender,
        uint256 amount,
        string destinationChain,
        string destinationAddress
    );
    event BridgeFundsOut(
        address token,
        address recipient,
        uint256 amount,
        string sourceChain,
        string sourceAddress
    );

    address public operator;

    constructor() {
        operator = msg.sender;
    }

    modifier onlyOperator() {
        require(msg.sender == operator, "Method only callable by operator");
        _;
    }

    function bridgeIn(
        address token,
        address sender,
        uint256 amount,
        string calldata destinationChain,
        string calldata destinationAddress
    ) external {
        IERC20(token).safeTransferFrom(sender, address(this), amount);

        emit BridgeFundsIn(
            token,
            sender,
            amount,
            destinationChain,
            destinationAddress
        );
    }

    function bridgeOut(
        address token,
        address recipient,
        uint256 amount,
        string calldata sourceChain,
        string calldata sourceAddress
    ) external onlyOperator {
        address sender = address(this);
        IERC20(token).approve(sender, amount);
        IERC20(token).safeTransferFrom(sender, recipient, amount);

        emit BridgeFundsOut(
            token,
            recipient,
            amount,
            sourceChain,
            sourceAddress
        );
    }

    function transferOut(
        address token,
        address recipient,
        uint256 amount
    ) external onlyOperator {
        address sender = address(this);
        IERC20(token).approve(sender, amount);
        IERC20(token).safeTransferFrom(sender, recipient, amount);
    }
}
