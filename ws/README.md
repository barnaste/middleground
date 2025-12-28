We expect that the sender's ID is known upon opening the web-socket, and
so does not need to be recorded on each message send.

We use one web-socket connection for each user for each channel. These
connections broadcast information over `conversation:{id}:messages`.
So, each thread handling a web-socket connection knows both the ID of
the user it's communicating with, and the ID of the channel it's sending
messages to.

# Message Use Cases
*NOTE*: The general idea in design is to ensure that the information is
committed to the database prior to being broadcasted to the users.

## SEND
Client sends the data
```json
{
    "type": "send",
    "payload": {
        "content": 
        "quotedId": // optional
    }
}
```
The server creates a new message and message atom in the DB, and 
retrieves the new messageId, to be sent back to the sender for future
use. Then, the server concurrently signals the new message to the 
recipient with the sender ID.

## EDIT
Client sends the data
```json
{
    "type": "edit",
    "payload": {
        "messageId": 
        "content": 
    }
}
```
The server creates a new message atom in the DB, and once completed
replies with an acknowledgement indicating that the message was success-
fully edited. Concurrently, the new message is sent to the recipient
with the sender ID.

## DELETE
Client sends the data
```json
{
    "type": "delete",
    "payload": {
        "messageId":
    }
}
```
Once recorded in the DB, the update is broadcasted to all users, and the
message becomes deleted, disappearing from the screen.

*NOTE*: timestamps are to be generated at the time of storage in the 
database. This way, we entirely skip the possibility of user tampering.

