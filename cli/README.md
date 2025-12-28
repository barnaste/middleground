# Creating a Command-Line Tool
The objective of this create is to act as a tool for the client to send
messages to a channel in place of a front-end. When we introduce features
such as sources, matchmaking, etc, we will expand this tool. Eventually,
we will do away with this altogether using an actual front-end, besides
usage for convenience in debugging.

For now, we expect the user to provide key information, such as the
channel they connect to. As consequence, it will temporarily be possible 
to have more than 2 users connected to a channel.

## Authentication
On launch, the tool should request the user's email, and have them
subsequently log in. This should be done by sending a request to /send-otp
and waiting for a code to be provided by the user, then sent to /verify-otp. 
- On error, continue to prompt for correct OTP.
- On success, information including user ID and refresh/access tokens
  is retrieved.

After completion of verification, we should prompt the user for the next
action. For now, the only possible action will be `c [conv_id]`, which
connects the user to the conversation/channel with the given ID. This 
should establish a web-socket connection by connecting to the /ws endpoint.
This should require strict authentication. For now, we will simply trust
the initial authentication.
- The /ws endpoint is expected to, as all other services, be defined in
  its own router, added to the main router under the `api_gateway` crate.

## Messaging
Once the user has successfully connected, a message should be printed
to indicate successful connection. The user has various options
1. send messages with `s [message content]`
2. reply to messages with `r [target ID] [message content]`
3. edit messages with `e [target ID] [message content]`
4. delete messages with `d [target ID]`
5. exit with `q`

For instance, the following should be a valid stream of input/output
```
==============
PLEASE SIGN IN
==============
email: my.name@mail.ca
otp: 314158
== WARNING: incorrect OTP ==
otp: 314159

==============
   WELCOME!
==============
> c 104
== Connecting to Channel 104... ==
s hello!
        [01][10][19] hello!
        [02][11][20] wow your so cool
r 11 *you're 
    [01][12][21][11] *you're
        [02][13][22] oh oopsies
        [02][11][23] wow you're so cool
d 12
            [01][12] == message deleted ==
q

> q
==============
   GOODBYE!
==============
```

Note that in the format `[id1][id2][id3][id4] message`, 
- `id1` refers to the sender's ID
- `id2` refers to the message's ID
- `id3` refers to the atom's ID, if present
- `id4` refers to the ID of the message replied to, if present
