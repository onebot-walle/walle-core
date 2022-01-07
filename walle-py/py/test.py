from asyncio.runners import run
import walle
import asyncio


async def main():
    await walle.run_onebot11_app()
    while True:
        await asyncio.sleep(1)

if __name__ == "__main__":
    print("this is a walle-py test")
    asyncio.run(main())
