from asyncio.runners import run
import walle
import asyncio


async def main():
    await walle.run_block_onebot11_app()

if __name__ == "__main__":
    asyncio.run(main())
