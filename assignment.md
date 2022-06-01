# Treasury Contract
Heavenland has introduced **Loaylty NFTs**, granting their holders a discount for the payments made in HTO.

- Example:
  - user pays 1,000 HTO
  - 10 HTO (1%) is Heavenland service fee
  - In time of the payment, user owns 5 Loaylty NFTs, granting 20% discount on service fee
  - user gets 2 HTO back
- Simply put, the discount works like a cash-back - the payer is charged in full and the discount is returned to him.

How many Loyalty NFTs user owns is detected by BE, which stores all user's payments. When the payment is detected, BE sends request to Solana blockchain to find out how many Loyalty NFTs the user owns.

BE knows how many HTO user should get. To send HTO back to user, we will introduce Treasury Contract, that will

- receive HTO with a parameter who can claim it (anyone can send HTO to treasury an tell which address can claim the HTO); if no address is given, transaction will fail
- allow claiming the gathered cash-backs (given address can claim only HTO for which it's eligible)
- provide info about how many HTO can be claimed and how many HTO has already been claimed by given address
- allow defining admin address(es) that will be able to claim the HTO addressed to arbitrary address (in case someone looses private key to his wallet so we can restore at least this HTO)

Interaction with Treasury Contract will be through Heavenmarket (in user account section), which is a vue.js web applicaton. The BE interacts with Treasury Contract only when adding an amount for an address to claim. 
Development

I'd like the development to be test-driven and all of the following to be created

- defining test scenarios (this is done by Heavenland)
- creating Solana program (in Rust) and CLI (in typescript) allowing interaction with the program. All test scenarios must be implemented and all of them must pass.
- both Solana program and CLI must be propperly documented, wich includes
  - documenting all methods (what they do and all parameters they are called with)
  - instructions how to run test scenarios and deploy the program
- there must be a binding of CLI to FE, which can we approach either by
  - Heavenland providing a simple vue.js skeleton
  - simple frontend (in pure javascript or vue.js) created as part of this job

## Test Scenarios

```
it("When program is deployed, there is only one Admin: Program creator",
it("Admin can add another Admin",
it("non-Admin cannot add another Admin",
it("If there is more than one Admin, Admin can remove another Admin",
it("Admin cannot remove himself as Admin",
```

```
describe("When sending x HTO without providing Receiver address",
  it("transaction fails", 
```

```
describe("When sending x HTO to Receiver",
  it("non-Receiver cannot claim HTO",
  it("Receiver can claim x HTO",
  it("Admin can claim x HTO providing Receiver address",
```

```
describe("When sending x HTO to Receiver, then y HTO to Receiver",
  it("Receiver can claim x+y HTO",
```

```
describe("When sending x HTO to ReceiverA and y HTO to ReceiverB",
  it("ReceiverA can claim x HTO",
  it("ReceiverB can claim y HTO",
```

```
describe("x HTO goes to Receiver, he claims, then y HTO goes to Receiver",
  it("Reciever has already claimed x HTO",
  it("Reciever can claim y HTO",
```