#pragma once

/* Generated with IDL v0.1.1 */


#include "packetfactory.h"

namespace RoseCommon {
namespace Packet {

class CliTest : public CRosePacket {
    public:
        static constexpr ePacketType PACKET_ID = ePacketType::PAKCS_TEST;
        CliTest();
        CliTest(CRoseReader reader);
        CliTest(CliTest&&) = default;
        CliTest& operator=(CliTest&&) = default;
        ~CliTest() = default;
        
        static constexpr size_t size();
        
        
        
        CliTest& set_test(const uint8_t);
        uint8_t get_test() const;
        
        
        static CliTest create(const uint8_t& test);
        static CliTest create(const uint8_t*);
        static std::unique_ptr<CliTest> allocate(const uint8_t*);
    
    protected:
        virtual bool pack(CRoseBasePolicy&) const override;
    
    private:
        uint8_t test;
};

}
}
