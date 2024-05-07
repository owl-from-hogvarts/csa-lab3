import pytest
import tempfile
import os


@pytest.mark.golden_test("tests/*.yml")
def test_golden(golden):
    with tempfile.TemporaryDirectory() as tmpdirname:
        source = os.path.join(tmpdirname, "source.asm")
        output = os.path.join(tmpdirname, "output.txt")
        target = os.path.join(tmpdirname, "source.json")
        input = os.path.join(tmpdirname, "input.txt")
        
        with open(source, "w", encoding="utf-8") as file:
            file.write(golden["source"])

        with open(input, "w", encoding="utf-8") as file:
            file.write(golden["input"])



        os.system(f"cd assembler && cargo run -- {source} {target}")
        with open(target, "r") as file:
            code = file.read()
            assert code == golden.out["machine_code"]
            

        os.system(f"cd cpu && cargo run -- {target} {input} >> {output}")
        with open(output, "r") as file:
            code = file.read()
            assert code == golden.out["output"]
    
        with open("cpu/cpu.log", "r", encoding="utf-8") as file:
            log = file.read()
            assert log == golden.out["out_log"]
