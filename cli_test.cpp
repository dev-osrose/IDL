#include "cli_test.h"

using namespace RoseCommon;
using namespace RoseCommon::Packet;


void CliTest::Test::set_a(const int a) {
    this->data.a = a;
}

int CliTest::Test::get_a() const {
    return data.a;
}

void CliTest::Test::set_b(const int b) {
    this->data.b = b;
}

int CliTest::Test::get_b() const {
    return data.b;
}

bool CliTest::Test::write(CRoseBasePolicy& writer) const {
    if (!writer.set_union(data)) {
        return false;
    }
    return true;
}

bool CliTest::Test::read(CRoseReader& reader) {
    if (!reader.get_union(data)) {
        return false;
    }
    return true;
}

constexpr size_t CliTest::Test::size() {
    return sizeof(data);
}


CliTest::CliTest() : CRosePacket(ePacketType::PAKCS_TEST) {}

CliTest::CliTest(CRoseReader reader) : CRosePacket(reader) {
}

CliTest CliTest::create() {
    CliTest packet;
    return packet;
}

CliTest CliTest::create(const uint8_t* buffer) {
    CRoseReader reader(buffer, CRosePacket::size(buffer));
    return CliTest(reader);
}

void CliTest::pack(CRoseBasePolicy& writer) const {
}

constexpr size_t CliTest::size() {
    size_t size = 0;
    return size;
}

