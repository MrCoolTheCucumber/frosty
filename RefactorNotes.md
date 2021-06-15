# Notes if I ever decide to do a full re-write
- Follow the model of each component owning its registers, rather than each 
component holding an `Rc<RefCell<Mmu>>`
- Make use of operator overloads, would be nice for custom register classes.
- Mark code as running in debug mode only: https://stackoverflow.com/questions/39204908/how-to-check-release-debug-builds-using-cfg-in-rust
- Statically generate opcode handling code rather than dynamic. Use macro's to help with repeating code.
(e.g. macro for Add A, r8 thats used for each register)
- Ask questions in the rust discord (or emudev rust section?), lots of smart people there