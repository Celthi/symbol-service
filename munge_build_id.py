import itertools

# from https://gist.github.com/luser/193572147c401c8a965c
def munge_build_id(build_id):
    '''
    Breakpad stuffs the build id into a GUID struct so the bytes are
    flipped from the standard presentation.
    '''
    b = list(map(''.join, list(zip(*[iter(build_id.upper())]*2))))
    return ''.join(itertools.chain(reversed(b[:4]), reversed(b[4:6]),
                                   reversed(b[6:8]), b[8:16])) + '0'


# print(munge_build_id('54a52c3a5c46353b3c3b5f64fc9406bb86f9708a'))
print(munge_build_id('e7013ddda03e9bcfd755837e0eb76350eded4bd1'))
# 3A2CA554465C3B353C3B5F64FC9406BB0

print(munge_build_id('9f9ffdfa6bf1398b70faaa1103599cc736b1c402'))
# FAFD9F9F-F16B-8B39-70FA-AA1103599CC70
