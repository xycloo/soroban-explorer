# Soroban Explorer
#### Utils + webapps to explore Soroban


## About
This workspace (will) provides crates with utils to explore the Futurenet, and (will) contains also web apps that use these crates to ease the process of exploring the Futurenet.

## Crates

### explorer-common
This crate (will) contains utils that web apps and other crates share in common, currenlty the crate provides utils to decode a soroban invocation transaction (also currently assumes one operation, this is on my TODO).

## Web Apps

### Soroban transaction explore
[Soroban-fiddle](https://github.com/leighmcculloch/soroban-fiddle) works really great, but when you already know your transaction hash it's better to not stream all operations and read all the respective transactions. However, when looking at soroban we can understand very little about the hostfunction invocation without manually decoding every inch of the evenlope, result, and meta XDR. This simple yew-built web app allows you to explore a transaction without having to decode anything: just paste the transaction id hash in the input field!

## Credits
Many design concepts where taken from [soroban-fiddle](https://github.com/leighmcculloch/soroban-fiddle).

