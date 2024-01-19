
## Before You Connect to Tokenisation
https://developerengine.fisglobal.com/apis/wpg/tokenisation

### What to consider before you add Tokenisation:
1. Choose your token format(s)
2. Choose shopper or merchant-related tokens
3. Decide whether to share tokens across merchant codes
4. Be aware of token expiry
5. Configure secure HTTP(S) URL
6. Sending batch orders
7. Migrate tokens from a previous provider
8. Compatibility with Account Updater
9. Recurring transaction decline codes
 

Tokenisable payment methods
Cards and wallets

You can tokenise these cards (optionally through the MASTERPASS-SSL wallet):

#### Cards	
AIRPLUS-SSL         DISCOVER-SSL
AMEX-SSL	        ECMC-SSL
CARTEBLEUE-SSL	    JCB-SSL
CB-SSL	            MAESTRO-SSL
DANKORT-SSL	        UATP-SSL
DINERS-SSL	        VISA-SSL


#### APMs
1. PayPal
2. SEPA Direct Debit


Choose token format(s)
You can choose from four token formats. Any tokens that you intend to share (across merchant codes) must all use the same format:

Alphanumeric token – this token uses an alternating sequence of upper case letters and numbers, and is always 15 digits long. The letters I and O are never used due to their similarity with numbers. Useful if you do not require a numeric token, and prefer the token to be clearly distinguished from card numbers.

Wrapped luhn fail token – this token consists of between 16 and 21 digits, and the first two digits are always 99. These tokens will not pass through luhn (modulus 10) checks. Useful if you want to avoid possible issues with regard to PCI DSS compliance checks, because tokens will not be mistaken for a PAN.

Wrapped luhn pass token – this token consists of between 16 and 21 digits, and the first two digits are always 99. These tokens will pass through luhn (modulus 10) checks. Useful if you have system validation that requires your token to be similar to a PAN.

Format preserving token – this token uses a part of the card number used to create it, and is always 20 digits long. The first two digits are always 99. The next six digits are the first six numbers on the card used. The final four digits are the last four numbers on the card used.

For example: 99123456xxxxxxxx7890. The digits marked as ‘x’ are randomly generated. The 18 digits which follow the 99 prefix (e.g. 123456xxxxxxxx7890) will fail a luhn check.

This is useful if you need your token to contain some information about the card associated with it, rather than getting information about the card from the separate obfuscated PAN already provided.

Warning
For the highest level of security we recommend using the maximum length of tokens your systems can accommodate (where the format allows variable length), and avoid use of the format preserving option unless you specifically require that functionality.
 

Shopper or merchant tokens
 

#### Which tokens to use:
Choose shopper tokens if you only intend to use tokens for your eCommerce channel
Choose merchant tokens is you intend to share tokens between our eCommerce and your Point of Sale (POS) channels (using Worldpay Total as your omni-channel solution)
 

Shopper tokens are created and used as part of a shopper account on a website, typically as part of an e-Commerce transaction. Shopper tokens can be used to store a shopper's card details, even when a payment has not been made (such as storing card details for later use). Additionally, they can be used in conjunction with 
Client Side Encryption


Merchant tokens are used when created as part of a POS transaction, where payments are made within the face-to-face area of your business and a shopper ID is not available. These tokens can also be created and used through the e-Commerce channel, enabling a holistic view of shopper activity. Merchant tokens can be used in conjunction with Client Side Encryption


### Token sharing
Depending on how your business is organised, you can create and use your tokens within a single merchant code, or across multiple merchant codes.

A merchant code is our terminology for a particular merchant account, and you can have one or more of these.

### Token expiry
In production, tokens are created with an initial life of 4 years. In Sandbox, tokens are created with an initial life of 7 days.

### Changing the token life

During your Tokenisation setup, you can have Worldpay change the default length of the token life by creating a new token group. This cannot be changed once you begin creating tokens. The maximum token life is 4 years (48 months). Speak to your Relationship Manager or Corporate Support Manager for more information.


### Recurring transaction decline codes
If you use tokenisation for recurring transactions, Visa have 'stop payment' decline codes that aim to reduce cardholder complaints caused by recurring transactions appearing on shopper bank statements after they have cancelled a recurring agreement. These codes are:

R1 - Revocation of authorization order
R3 - Revocation of all authorization order
Both response codes are used by issuers to decline recurring authorisation requests.

In our authorisation response, we map these VISA codes to these numeric codes:

Copy Copied!
<ISO8583ReturnCode code="973" description="Revocation of Authorization Order"/>
<ISO8583ReturnCode code="975" description=" Revocation of All Authorizations Order"/>
What should I do if I receive one of these decline codes?

You must ensure that your systems can process these decline codes. This may require you to make changes to your systems.

If you receive a R1 or R3 decline code, you must stop any further authorisation attempts against the card used and notify the cardholder.


