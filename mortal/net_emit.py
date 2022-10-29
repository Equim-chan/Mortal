from io import BytesIO
import torch
import struct


def send_msg(conn, msg, packed=False):
    if packed:
        tx = msg
    else:
        buf = BytesIO()
        torch.save(msg, buf)
        tx = buf.getvalue()
    conn.sendall(struct.pack('<Q', len(tx)))
    conn.sendall(tx)

def recv_msg(conn, map_location=torch.device('cpu')):
    rx = recv_binary(conn, 8)
    (size,) = struct.unpack('<Q', rx)
    rx = recv_binary(conn, size)
    return torch.load(BytesIO(rx), map_location=map_location)

def recv_binary(conn, size):
    assert size > 0
    ret = bytearray(size)
    buf = memoryview(ret)

    while len(buf) > 0:
        n = conn.recv_into(buf)
        if n == 0:
            raise UnexpectedEOF()
        buf = buf[n:]
    return bytes(ret)

class UnexpectedEOF(Exception):
    def __init__(self):
        super().__init__('unexpected EOF')