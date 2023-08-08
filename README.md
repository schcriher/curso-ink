# Instalación del entorno de derarrollo (Linux)

## Rust & Cargo

- Web: https://www.rust-lang.org/tools/install
- Acción: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Ink!

- Webs:
  - https://use.ink/getting-started/setup/
  - https://github.com/paritytech/cargo-contract
- Acciones:
  - `rustup component add rust-src`
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install --force cargo-contract`

## Substrate contracts node

- Webs:
  - https://github.com/paritytech/substrate-contracts-node
  - https://github.com/paritytech/substrate-contracts-node/releases
- Acciones: descargar `substrate-contracts-node-linux.tar.gz`

# Proyecto

## Software utilizado:

| Software                 | Versión                                |
| ------------------------ | -------------------------------------- |
| rustup                   | 1.26.0 (5af9b9484 2023-04-05)          |
| rustc                    | 1.70.0 (90c541806 2023-05-31)          |
| cargo                    | 1.70.0 (ec8a8a0ca 2023-04-25)          |
| cargo-contract-contract  | 3.0.1-unknown-x86_64-unknown-linux-gnu |
| substrate-contracts-node | 0.30.0-72e68577688                     |

## Inicialización del proyecto

```Bash
cargo contract new flipper
cd flipper

cargo test --package flipper --lib -- flipper::tests --nocapture

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

- [ ] Incluir a los contribuyentes con su reputación asociada (usar vectores).
- [ ] Incluir una cuenta administradora, que podrá agregar/eliminar contribuyentes.

**Mensajes:**

- [ ] Agregar/Eliminar contribuyente
- [ ] Votar (sólamente un contribuyente puede votar a otro)
- [ ] Consultar reputación de contribuyente
