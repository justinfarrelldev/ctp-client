# Server is Authoritative

The server should be the authoritative source-of-truth for the games, whether singleplayer or multiplayer. If the client disagrees with the server, the client must display a "resynchronizing" message and update its full state from the server.

# Client is Optimistic

The client should optimistically start moving units before confirmation of a valid move from the server in cases where that is ambiguous (such as obscured terrain). The client must have the request to move / do an action / whatever validated by the server - if there is a schism, the client must revert.

# Anti-cheating

The client should never receive more terrain information than they can see - unknown tiles outside of view must remain totally obscured.

# Hybrid play modes must always be kept in mind

Hybrid will be the main multiplayer mode - keep it in the forefront of your mind.
