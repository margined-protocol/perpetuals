# Margined Protocol v1 Design Doc

This document details the design of the Margined Protocol Perpetuals (MPP) v1.

The additional features of the Perpetuals v1 are:

* multi-collateral
* on-chain governance

In this document we will detail how the new features will integrate with the existing MPP v0 and key design decisions.

## Multi-Collateral

The ability to have multiple forms of eligible collateral is a key feature for on-boarding additional users to Margined Protocol. By allowing multiple forms of eligible collateral, as opposed to a single stablecoin, a wider set of users will be able create positions. However, by allowing multiple forms of collateral a number questions and design decisions must be answered.

* How will positions be valued?
* How will collateral be valued?
    * Can I get liquidated due to depreciation of my collateral?
* Can multi-collateral positions be cross margined?
* How will multi-collateral work?
    * Depositing and Withdrawing Margin
    * Open/Close position
    * Fees
    * Profit and Loss
    * Funding rate
* What parts of the code be affected?
* Other problems?
* more questions
    * how will profits be paid?
    * how will collateral liquidations be handled?

### How will positions be valued?

All perpetual contracts listed by on MPP will be quoted in USD. Using USD as a quote currency provides the common denominator for portfolio valuation of the end users.

In the future MPP may list contracts using non-USD quote assets, however this is currently out-of-scope.

If users are able to take positions with assets different to the quote token then collateral must be valued in the quote token.

### How will collateral be valued?

As positions are to be valued in the quote asset of MPP, typically USD, collateral requirements or margin must be valued in the quote asset. Therefore, when a user tries to open a position using non quote asset collateral we must not only perform the calculation of the margin requirements wrt to the position, but also value the collateral in the quote-asset.

However, each form of eligible collateral will also be assigned a risk factor, `0 < risk_factor <= 1`, which will be used to calculate the value of the collateral in the quote asset. Where,

```
collateral_value = exchange_rate * amount * risk_factor
```

The risk factor should try to minimise the effects of volatility on the position of a user. So that if the collateral of a user devalues significantly the position is not immediately liquidated.

If the non quote asset collateral of a user depreciates and falls below the maintenance and liquidation ratios the position can be liquidated.

### Can multi-collateral positions be cross-margined?

No cross-margining is only available for individual collateral types. For example if I have a short position using a non-quote asset collateral and a long position use the quote asset as collateral (or even an alternative non-quote asset collateral) the positions will be treated in isolation, including profits and losses.

However, for positions across different products that use the same collateral cross-margining is applied.

### How will multi-collateral work?

Adding multi-collateral will effect all parts of the protocol where tokens are transfered. This includes:

* Depositing or withdrawing margin
* Opening and closing positions
* Profits and losses
* Paying fees
* Paying funding rate
* Liquidation

#### Deposition of withdrawing margin

As multiple collateral contracts are not cross-margined per collateral type therefore deposit and withdraw of collateral need no major changes, a user will maintain a single position per contract per collateral.

When depositing non-quote asset collaterals the user will simply add the value to their existing margin. However, their buying power will only increase by the latest rate multiplied by the relevant risk factor.

During a withdrawal a user may only remove as much collateral so that the margin ratio it greater than the the initial margin ratio.

#### Opening and closing positions

Scenarios:

* open new position 

## On-Chain Governance

* reward payouts?
* what can be voted on?