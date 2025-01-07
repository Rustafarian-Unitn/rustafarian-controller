# List and description of the tests to be done
Each test is carried out with three different topologies, the first with just a server and a client, the second with 2 servers and 2 clients to see the interactions between them and the third with a much more complex network. These will be called topology_1.toml, topology_2.toml, topology_3.toml and will be in the configurations folder
## Test for content server (content_test.rs):
   - Initialization with Flood Request and Response control and correctness of the topology
   - Request **Server Type** 
   - Request **File List**
   - Request **File Text**
   - Request **File Media**
   - Combined Request **File Text e Media**
   - **Drop Packet** relating to the request for a text file from the client
   - **Drop Packet** relating to the response for a text file from the server
   - **Crash Drone** and sending an error in routing, with related flood_request and resend of the packet (client pov)
   - **Crash Drone** and sending an error in routing, with related flood_request and resend of the packet (server pov)
## Test for chat server (chat_test.rs):
   - Initialization with Flood Request and Response control and correctness of the topology