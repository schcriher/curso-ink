# Instalación del entorno de derarrollo (Linux)

## Rust & Cargo

- Web: https://www.rust-lang.org/tools/install
- Acción: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Ink!

- Webs:
  - https://use.ink/getting-started/setup/
  - https://github.com/paritytech/cargo-contract
- Acciones: `cargo-contract <= 3.0.1` es incompatible con `rustc >= 1.70.0` y `cargo-contract = 4.0.0` está en beta:
  - `rustup install 1.69`
  - `rustup default 1.69`
  - `rustup component add rust-src --toolchain 1.69`
  - `rustup target add wasm32-unknown-unknown --toolchain 1.69`
  - `cargo install --force --version 3.0.1 cargo-contract`

## Substrate contracts node

- Webs:
  - https://github.com/paritytech/substrate-contracts-node
  - https://github.com/paritytech/substrate-contracts-node/releases
- Acciones: descargar `substrate-contracts-node-linux.tar.gz`

## Nodos

- Local:
  - ejecutar `substrate-contracts-node`
  - Webs:
    - https://contracts-ui.substrate.io
    - https://polkadot.js.org/apps/

# Proyecto

## Software utilizado:

| Software                 | Versión                                |
| ------------------------ | -------------------------------------- |
| rustup                   | 1.26.0 (5af9b9484 2023-04-05)          |
| rustc                    | 1.69.0 (84c898d65 2023-04-16)          |
| cargo                    | 1.69.0 (6e9a83356 2023-04-12)          |
| cargo-contract           | 3.0.1-unknown-x86_64-unknown-linux-gnu |
| substrate-contracts-node | 0.30.0-72e68577688                     |

## Inicialización del proyecto

```Bash
cargo contract new flipper
cd flipper

cargo test --package flipper --lib -- flipper::tests --nocapture
cargo contract build --target wasm

git init
git add .
git commit -m "class #1"
git branch -M master
git remote add origin git@github.com:schcriher/curso-ink.git
git push -u origin master
```

## Avance del proyecto

### Objetivo

Armar una organización que premie a sus contribuyentes según su reputación.

### Clase 1

- [x] Configurar el entorno de desarrollo local
- [x] Generar un contrato flipper
- [x] Generar un repositorio git personal para el seguimiento del trabajo práctico
- [x] Subir el código del contrato generado
- [x] Compartir el repositorio en el canal de discord para el trackeo del mismo

### Clase 2

Modificar el smart contract para empezar a darle forma a nuestra organización:

**Storage:**

- [x] Incluir a los contribuyentes con su reputación asociada (usar vectores).
- [x] Incluir una cuenta administradora, que podrá agregar/eliminar contribuyentes.

**Mensajes:**

- [x] Agregar/Eliminar contribuyente
- [x] Votar (sólamente un contribuyente puede votar a otro)
- [x] Consultar reputación de contribuyente

**Notas:** para esta etapa del desarrollo se asumen las siguientes condiciones:

- La "reputación" es la suma de votos que tiene un contribuyente
- Un contribuyente puede votar infinitamente a otros contribuyentes
