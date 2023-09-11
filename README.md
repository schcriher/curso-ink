# DAO: Decentralized Autonomous Organization

## Instalación del entorno de derarrollo (GNU/Linux)

### Rust & Cargo

- Web: https://www.rust-lang.org/tools/install
- Acción: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Ink!

- Webs:
  - https://use.ink/getting-started/setup/
  - https://github.com/paritytech/cargo-contract
- Acciones:
  - `rustup install 1.72`
  - `rustup default 1.72`
  - `rustup component add rust-src --toolchain 1.72`
  - `rustup target add wasm32-unknown-unknown --toolchain 1.72`
  - `cargo install --force --version 3.2.0 cargo-contract`

### Node

- Webs:
  - https://github.com/paritytech/substrate-contracts-node
  - https://github.com/paritytech/substrate-contracts-node/releases
- Acciones:
  - Descargar `substrate-contracts-node-linux.tar.gz`
  - Colocar `substrate-contracts-node` en una carpeta del PATH del sistema

---

## Proyecto

### Software utilizado:

| Software                 | Versión                                |
| ------------------------ | -------------------------------------- |
| rustup                   | 1.26.0 (5af9b9484 2023-04-05)          |
| rustc                    | 1.72.0 (5680fa18f 2023-08-23)          |
| cargo                    | 1.72.0 (103a7ff2e 2023-08-15)          |
| cargo-contract           | 3.2.0-unknown-x86_64-unknown-linux-gnu |
| substrate-contracts-node | 0.31.0-c8863fe08b7                     |

### Inicialización del proyecto (clase #1)

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

---

## Ejecución de pruebas

```Bash
cargo test --package organization --lib -- organization::tests --nocapture
```

## Compilación

```Bash
cargo contract build --target wasm --manifest-path contracts/organization/Cargo.toml
# Resultado en: target/ink/nft/nft.contract

cargo contract build --target wasm --manifest-path contracts/nft/Cargo.toml
# Resultado en: target/ink/organization/organization.contract
```

## Ejecución local

- Ejecutar `substrate-contracts-node`
- Webs:
  - https://contracts-ui.substrate.io
  - https://polkadot.js.org/apps/

---

## Avance del proyecto

**Objetivo:** Armar una organización que premie a sus contribuyentes según su reputación.

<br/>

### 📝 Clase 1

- [x] Configurar el entorno de desarrollo local
- [x] Generar un contrato flipper
- [x] Generar un repositorio git personal para el seguimiento del trabajo práctico
- [x] Subir el código del contrato generado
- [x] Compartir el repositorio en el canal de discord para el trackeo del mismo

<br/>

### 📝 Clase 2

Modificar el smart contract para empezar a darle forma a nuestra organización:

**Storage:**

- [x] Incluir a los contribuyentes con su reputación asociada (usar vectores).
- [x] Incluir una cuenta administradora, que podrá agregar/eliminar contribuyentes.

**Mensajes:**

- [x] Agregar/Eliminar contribuyente
- [x] Votar (sólamente un contribuyente puede votar a otro)
- [x] Consultar reputación de contribuyente

<br/>

> **Notas:** _para esta etapa del desarrollo se asumen las siguientes condiciones:_
>
> - _La "reputación" es la suma de votos que tiene un contribuyente_
> - _Un contribuyente puede votar infinitamente a otros contribuyentes_

<br/>

### 📝 Clase 3

- [x] Modificar el storage para utilizar Mappings en lugar de Vectores
- [x] Modificar lógica para que el poder de voto se corresponda con la reputación del contribuyente
      (mayor reputación → mayor poder de voto)
- [x] Emitir un evento por cada voto
- [x] Agregar los siguientes controles:
  - El único que puede agregar o eliminar contribuyentes es el Admin
  - Los únicos que pueden votar son los contribuyentes registrados
  - La reputación es privada. Cada contribuyente puede consultar únicamente la propia

<br/>

> **Notas:** _Proyecto renombrado a "Organization", además se asumen las siguientes condiciones:_
>
> - _Hasta tener una definición mas completa de la lógica de negocio:_
>
>   - _Se mantiene la emisión ilimitada de votos_
>   - _Se mantiene la emisión de votos positivos unicamente_
>   - _El poder de voto se divide en dos categorías:_
>
>     - _contribuyentes con menos de 10 votos: suman 1 a la reputación con cada voto_
>     - _contribuyentes con 10 o mas votos: suman 2 a la reputación con cada voto_
>
> - _El evento emitido en la votación contiene las direcciones del votante y el votado_

<br/>

### 📝 Clase 4

- [x] Crear un contrato PSP34 (Utilizar Templates de OpenBrush) que sirva de certificado de votación
- [x] Transferir al contribuyente un NFT que certifique su voto
- [x] Definir un trait que represente el comportamiento de votación e implementarlo en el contrato:
  - Votar
  - Obtener reputación/votos de un contribuyente

<br/>

> **Problema de dependencias:**
>
> - Conflicto:
>   - `subxt-codegen v0.31.0` (paquete no manejado directamente) depende de `rustc >= 1.70`
>   - `cargo-contract v3.2.0` (paquete nuevo) soporta `rustc > 1.69`
> - Acciones:
>   - Pasos de la sección de instalacion de [Ink!](#ink) (actualizado)
>
> **Notas:**
>
> - Al interactuar con un segundo contrato ya no es posible realizar pruebas unitarias con datos falsos como se estaba haciendo hasta ahora. El archivo `ink_env-4.3.0/src/engine/off_chain/impls.rs` en su función `instantiate_contract` (línea 488) lo define como "no implementado" con el mensaje _"off-chain environment does not support contract instantiation"_, por lo tanto se eliminan todas las pruebas unitarias (quedan en el historial de git).

<br/>

### 📝 Clase 5

_Clase teórica sobre Chain Extensions, sin cambios en el proyecto._

<br/>

### 📝 Clase 6

- [ ] Agregar tests de integración y E2E al código de la clase #4. No hace falta que cubran el 100% del código.
